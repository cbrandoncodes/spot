# Task 03 — Onboarding State-Machine Regression Suite

## Timebox and outcome

**Target: 25–27 minutes. Do not exceed 30 minutes.**

Build a reusable Postman collection that proves the authenticated onboarding
workflow behaves as one state machine across HTTP and PostgreSQL:

```text
revision chaining -> validation boundaries -> completion -> idempotent retry
                  -> completed-draft edit -> next immutable profile revision
```

This task validates the schedule model from Task 1 and completion semantics from
Task 2. It does not change application behavior.

## Deliverables

```text
postman/Spot-Onboarding-Regression.postman_collection.json
postman/Spot-Local.postman_environment.example.json
postman/data/schedule-boundaries.json
rust/docs/postman.txt
```

Never export or commit a real OTP, email address, session token, or database URL.

## Prepare before recording

- Disable Postman AI and Cursor AI.
- Start the API with `cargo watch -x 'run --bin api'`.
- Obtain a valid `spot_token` before recording; OTP entry is not the task.
- Use a completed test account so editing a group can start a repeatable run.
- Keep Postman, Cursor, and pgAdmin minimized or empty at recording start.
- Confirm Tasks 1 and 2 pass their checks.

Create a private working environment with:

```text
base_url                 = http://127.0.0.1:8080
spot_token               = <private current token>
onboarding_revision      =
stale_revision           =
completion_revision      =
first_profile_revision   =
second_profile_revision  =
user_id                   =
```

## Recording plan

### 1. Opening and contract map — 1.5 minutes

Show the approved Goal document and state:

> Create a repeatable integration suite that verifies authentication, revision
> propagation, Task 1 schedule invariants, Task 2 completion retries, persistence,
> and re-editing as a single workflow.

In Postman, create the collection folders:

```text
00 Session
01 Complete onboarding
02 Revision conflicts
03 Schedule boundaries
04 Completion retry
05 Re-edit and re-complete
```

Reason: folders make dependencies visible without turning unrelated endpoints
into one opaque runner sequence.

### 2. Add collection-level authentication — 2 minutes

Set collection Authorization to `Bearer Token` with:

```text
{{spot_token}}
```

Add a collection-level pre-request guard:

```javascript
const required = ["base_url", "spot_token"];

for (const name of required) {
  const value = pm.environment.get(name);
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
}
```

Create `00 Session / Authenticated identity`:

```http
GET {{base_url}}/api/me
```

Post-response script:

```javascript
pm.test("Authenticated identity is available", function () {
  pm.response.to.have.status(200);
  const body = pm.response.json();
  pm.expect(body.id).to.be.a("string").and.not.empty;
  pm.environment.set("user_id", body.id);
});
```

Reason: every later result is meaningless if the runner silently uses an invalid
or expired session.

### 3. Build reusable response assertions — 3 minutes

Create `00 Session / Load onboarding draft`:

```http
GET {{base_url}}/api/onboarding
```

Use this post-response script on every successful draft GET/PATCH request:

```javascript
const onboardingSchema = {
  type: "object",
  required: [
    "questionnaireVersion", "revision", "currentGroup",
    "completedAt", "groups", "answers"
  ],
  properties: {
    questionnaireVersion: { type: "integer", minimum: 1 },
    revision: { type: "integer", minimum: 0 },
    currentGroup: { type: "string" },
    completedAt: { type: ["string", "null"] },
    groups: { type: "array", minItems: 4, maxItems: 4 },
    answers: { type: "object" }
  }
};

pm.test("Draft response contract is stable", function () {
  pm.response.to.have.status(200);
  pm.response.to.have.jsonSchema(onboardingSchema);
});

const body = pm.response.json();
pm.environment.set("onboarding_revision", body.revision);
```

For negative requests, use this shared problem assertion and adjust the expected
status/code variables per request:

```javascript
const body = pm.response.json();

pm.test("Problem response is structured", function () {
  pm.response.to.have.status(Number(pm.variables.get("expected_status")));
  pm.expect(body).to.include.keys("type", "title", "status", "detail", "code");
  pm.expect(body.code).to.equal(pm.variables.get("expected_code"));
});
```

Reason: schema assertions detect contract regressions that status-only tests miss.

### 4. Chain the happy-path groups — 5 minutes

Before the first PATCH, copy the loaded revision into the stale marker:

```javascript
pm.environment.set(
  "stale_revision",
  Number(pm.environment.get("onboarding_revision"))
);
```

Create these four requests. Each uses the current `{{onboarding_revision}}` and
the reusable successful-draft script, so every response supplies the next token.

`01 Complete onboarding / Goals and experience`:

```http
PATCH {{base_url}}/api/onboarding/groups/goals-experience
```

```json
{
  "revision": {{onboarding_revision}},
  "primaryGoal": "build_muscle",
  "secondaryGoals": ["get_stronger"],
  "experienceLevel": "beginner",
  "currentActivityLevel": "lightly_active"
}
```

`01 Complete onboarding / Schedule setup`:

```http
PATCH {{base_url}}/api/onboarding/groups/schedule-setup
```

```json
{
  "revision": {{onboarding_revision}},
  "daysPerWeek": 3,
  "minimumDurationMinutes": 30,
  "preferredDurationMinutes": 45,
  "maximumDurationMinutes": 60,
  "trainingEnvironment": "gym",
  "availableEquipment": ["dumbbells", "adjustable_bench"],
  "preferredWeekdays": ["monday", "wednesday", "friday"]
}
```

`01 Complete onboarding / Training preferences`:

```http
PATCH {{base_url}}/api/onboarding/groups/training-preferences
```

```json
{
  "revision": {{onboarding_revision}},
  "preferredWorkoutTypes": ["hypertrophy", "strength"],
  "dislikedExercises": [{
    "exerciseCode": "burpee",
    "preference": "never_include"
  }],
  "coachingStyle": "balanced",
  "startingIntensity": "gentle",
  "consistencyBarrier": "Variable work schedule"
}
```

`01 Complete onboarding / Movement and wellness`:

```http
PATCH {{base_url}}/api/onboarding/groups/movement-wellness
```

```json
{
  "revision": {{onboarding_revision}},
  "movementConstraints": [],
  "accessibilityNeeds": null,
  "heightMm": null,
  "weightGrams": null,
  "unitSystem": "metric",
  "acknowledgeSensitiveData": false
}
```

Add to the final PATCH script:

```javascript
const body = pm.response.json();
pm.test("All onboarding groups are complete", function () {
  pm.expect(body.groups.every(group => group.status === "complete")).to.equal(true);
});
pm.environment.set("completion_revision", body.revision);
```

Reason: propagating the server revision proves the client respects optimistic
concurrency instead of assuming sequential numbers.

### 5. Prove conflicts and boundary validation — 4 minutes

Create `02 Revision conflicts / Reject stale schedule update` using
`{{stale_revision}}` and otherwise valid schedule data. Set local variables:

```javascript
pm.variables.set("expected_status", 409);
pm.variables.set("expected_code", "onboarding_revision_conflict");
```

Create `postman/data/schedule-boundaries.json`:

```json
[
  {
    "name": "minimum exceeds preferred",
    "days": 3, "minimum": 60, "preferred": 45, "maximum": 90,
    "weekdays": ["monday", "wednesday", "friday"],
    "expectedField": "minimumDurationMinutes"
  },
  {
    "name": "preferred exceeds maximum",
    "days": 3, "minimum": 30, "preferred": 90, "maximum": 60,
    "weekdays": ["monday", "wednesday", "friday"],
    "expectedField": "preferredDurationMinutes"
  },
  {
    "name": "frequency exceeds weekdays",
    "days": 4, "minimum": 30, "preferred": 45, "maximum": 60,
    "weekdays": ["monday", "wednesday", "friday"],
    "expectedField": "preferredWeekdays"
  }
]
```

Create `03 Schedule boundaries / Reject inconsistent schedule`:

Pre-request script:

```javascript
pm.variables.set(
  "weekdays_json",
  JSON.stringify(pm.iterationData.get("weekdays"))
);
```

```json
{
  "revision": {{onboarding_revision}},
  "daysPerWeek": {{days}},
  "minimumDurationMinutes": {{minimum}},
  "preferredDurationMinutes": {{preferred}},
  "maximumDurationMinutes": {{maximum}},
  "trainingEnvironment": "gym",
  "availableEquipment": ["dumbbells"],
  "preferredWeekdays": {{weekdays_json}}
}
```

Test script:

```javascript
const body = pm.response.json();
pm.test(`${pm.iterationData.get("name")} is rejected`, function () {
  pm.response.to.have.status(422);
  pm.expect(body.code).to.equal("invalid_onboarding");
  pm.expect(body.field).to.equal(pm.iterationData.get("expectedField"));
});
```

Run only this request with Collection Runner and the JSON data file. Show all
three rows passing.

Reason: a data-driven boundary matrix is repeatable and materially stronger than
manually changing a payload during the recording.

### 6. Complete, retry, and verify persistence — 4 minutes

Create `04 Completion retry / Complete onboarding`:

```http
POST {{base_url}}/api/onboarding/complete
```

```json
{ "revision": {{completion_revision}} }
```

Post-response script:

```javascript
pm.test("Completion succeeds", function () {
  pm.response.to.have.status(200);
});

const body = pm.response.json();
pm.expect(body.profile.scheduleSetup.minimumDurationMinutes).to.equal(30);
pm.expect(body.profile.scheduleSetup.preferredDurationMinutes).to.equal(45);
pm.expect(body.profile.scheduleSetup.maximumDurationMinutes).to.equal(60);
pm.environment.set("first_profile_revision", body.revision);
```

Duplicate the request as `Retry same completion`. Keep the same revision and add:

```javascript
pm.test("Completion replay returns the original profile", function () {
  pm.response.to.have.status(200);
  pm.expect(pm.response.json().revision)
    .to.equal(Number(pm.environment.get("first_profile_revision")));
});
```

Create `GET {{base_url}}/api/fitness-profile/me` and assert the stored schedule
matches the completion response.

Reason: this verifies Task 2 through the public contract and Task 1 through the
normalized persisted representation.

### 7. Re-edit and create the next immutable revision — 3 minutes

Fetch `/api/onboarding`, save its revision, then PATCH schedule setup with:

```json
{
  "revision": {{onboarding_revision}},
  "daysPerWeek": 3,
  "minimumDurationMinutes": 35,
  "preferredDurationMinutes": 50,
  "maximumDurationMinutes": 70,
  "trainingEnvironment": "gym",
  "availableEquipment": ["dumbbells", "cable_station"],
  "preferredWeekdays": ["monday", "wednesday", "friday"]
}
```

Assert `completedAt` is `null`, save the new revision as `completion_revision`,
then complete again. Use:

```javascript
const body = pm.response.json();
const previous = Number(pm.environment.get("first_profile_revision"));

pm.test("Re-completion creates the next profile revision", function () {
  pm.response.to.have.status(200);
  pm.expect(body.revision).to.equal(previous + 1);
  pm.expect(body.profile.scheduleSetup.preferredDurationMinutes).to.equal(50);
});

pm.environment.set("second_profile_revision", body.revision);
```

In pgAdmin, verify both snapshots remain:

```sql
SELECT profile_revision, schema_version, created_at
FROM fitness_profile_snapshots
WHERE user_id = '<TEST_USER_UUID>'::uuid
ORDER BY profile_revision DESC
LIMIT 5;
```

Reason: editing a completed draft must create a new immutable history entry, not
rewrite the evidence for the earlier recommendation input.

### 8. Export safely and update documentation — 2 minutes

Export the collection to `postman/`. Export a separate example environment only
after clearing every value:

```json
{
  "base_url": "http://127.0.0.1:8080",
  "spot_token": "",
  "onboarding_revision": "",
  "stale_revision": "",
  "completion_revision": "",
  "first_profile_revision": "",
  "second_profile_revision": "",
  "user_id": ""
}
```

Update `rust/docs/postman.txt` with the collection/data-file locations and runner
order. In Cursor, search the exports before committing:

```bash
rg -n 'Bearer [A-Za-z0-9._-]+|spot_token.*[^:]$|@' postman
git diff --check
git diff --stat
```

Inspect every match; the search is a safety check, not an automatic guarantee.

### 9. Final run and epilogue — 1.5 minutes

Run the complete collection once with the private environment and show the
Postman summary. End with:

- authenticated workflow chained without hard-coded revisions;
- Task 1 invalid schedules rejected by field;
- Task 2 retry returned the same profile revision;
- re-editing produced the next immutable snapshot;
- exported artifacts contain no credentials.

State that the approved Goal is complete.

## Timing and compliance

| Phase | Time |
| --- | ---: |
| Opening and structure | 1.5 min |
| Collection authentication | 2 min |
| Shared assertions | 3 min |
| Happy-path chaining | 5 min |
| Conflicts and data-driven boundaries | 4 min |
| Completion and persistence | 4 min |
| Re-edit and snapshot verification | 3 min |
| Safe export and documentation | 2 min |
| Final run and epilogue | 1.5 min |
| **Total** | **26 min** |

Postman should remain the main application for at least 15 minutes. Keep typing
below 35% and each uninterrupted typing segment below 45 seconds. Shorten runner
waiting or navigation if delayed; do not omit unhappy-path assertions, immutable
snapshot proof, export sanitization, or the epilogue.
