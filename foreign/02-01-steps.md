# Task 02-01 — Reproduce and Diagnose Onboarding Completion Concurrency

## Outcome and timebox

**Target: 19–21 minutes. Hard maximum: 22 minutes.**

Produce a deterministic, reusable diagnosis of this existing Rust/PostgreSQL
defect without changing production behavior:

```text
two completion requests + same valid revision -> one 200 and one 409
database result                              -> one committed profile effect
root cause                                   -> revision predicate is evaluated around a row lock
```

This is the first task in the chain. Task 02-02 must reuse its Postman request,
SQL probes, observed evidence, and acceptance criteria.

## Deliverables

```text
postman/Spot-Onboarding-Concurrency-Reproducer.postman_collection.json
rust/diagnostics/onboarding-completion-locks.sql
rust/diagnosis/concurrency-diagnosis.md
```

Production Rust files are inputs and remain unchanged. The material work is an
executable concurrency reproducer plus a technical diagnosis grounded in live
HTTP and PostgreSQL evidence.

## Preparation outside the recording

- Disable AI assistants in Cursor and Postman.
- Have a valid private `spot_token`, `base_url`, and test-user UUID available.
- Start with a completed onboarding draft containing all four answer groups.
- Ensure `POST /api/onboarding/reopen` is available for resetting the test state.
- Start with the blank `rust/diagnosis/concurrency-diagnosis.md` template and a
  minimal Postman collection shell.
- Leave apps empty or minimized before recording; open inputs on camera.

Never expose or export the token, email, OTP, or database URL.

## Recording topology

```text
Cursor LOOKUP/RUN
  -> Postman CREATE/REPRODUCE
  -> pgAdmin LOOKUP/LOCK INSPECTION
  -> Cursor LOOKUP/DIAGNOSE
  -> Cursor CREATE DIAGNOSTIC OUTPUTS
```

Cursor remains the main app for roughly 10–11 minutes.

## Recording steps

### 1. Opening and baseline — 1.5 minutes

State:

> Spot is a Rust backend for a guided workout and social fitness platform. I am
> producing a deterministic diagnosis of a timing-dependent onboarding completion
> failure. The output will let the next engineering task implement and verify a
> fix from measured evidence rather than assumption.

In Cursor, show the clean baseline and run:

```bash
cd rust
cargo check --bin api
cargo test modules::fitness_profile
cargo run --bin api
```

Keep the API running in the integrated terminal. Reason: establish that the
defect exists in otherwise compiling code.

### 2. Inspect the request path — 2.5 minutes

Use Rust Analyzer navigation—not a broad text dump—to trace:

```text
complete_onboarding
FitnessProfileService::complete
FitnessProfileRepository::complete
RepositoryError::RevisionConflict
```

Record these observations in the diagnosis document:

- the service has a completed-draft retry fast path;
- the repository starts a PostgreSQL transaction;
- its `FOR UPDATE` query also filters by the submitted revision;
- profile, owned rows, snapshot, and draft update commit atomically.

Do not claim the revision predicate is the cause yet; the database experiment
must prove the timing behavior first.

### 3. Reset and capture a completable revision — 2 minutes

In Postman:

```http
GET {{base_url}}/api/onboarding
Authorization: Bearer {{spot_token}}
```

Save the current revision:

```javascript
const body = pm.response.json();
pm.environment.set("onboarding_revision", body.revision);
pm.environment.set("test_user_id", body.userId || pm.environment.get("test_user_id"));
```

If `completedAt` is populated, call:

```http
POST {{base_url}}/api/onboarding/reopen
Authorization: Bearer {{spot_token}}
Content-Type: application/json
```

```json
{ "revision": {{onboarding_revision}} }
```

Save the returned revision as `completion_revision`:

```javascript
const body = pm.response.json();
pm.test("Draft is reopened", function () {
  pm.response.to.have.status(200);
  pm.expect(body.completedAt).to.equal(null);
});
pm.environment.set("completion_revision", body.revision);
```

Reason: both concurrent commands must use the same known-valid revision.

### 4. Hold the coordination row — 2 minutes

In pgAdmin Query Tool session A, substitute the UUID and run:

```sql
BEGIN;

SELECT user_id, revision, current_group, completed_at
FROM onboarding_drafts
WHERE user_id = '<TEST_USER_UUID>'::uuid
FOR UPDATE;
```

Leave the transaction open. Record the returned revision and verify it equals
`completion_revision`.

Reason: this external lock queues both API transactions before either can win,
making the race deterministic without adding delay code to the application.

### 5. Fire two genuinely concurrent requests — 2.5 minutes

Create `Concurrent onboarding completion` in Postman. Its visible request may be:

```http
GET {{base_url}}/health/live
```

Add this post-response script:

```javascript
const completion = {
  url: `${pm.environment.get("base_url")}/api/onboarding/complete`,
  method: "POST",
  header: {
    Authorization: `Bearer ${pm.environment.get("spot_token")}`,
    "Content-Type": "application/json"
  },
  body: {
    mode: "raw",
    raw: JSON.stringify({
      revision: Number(pm.environment.get("completion_revision"))
    })
  }
};

const responses = [];

function collect(error, response) {
  pm.expect(error).to.equal(null);
  responses.push(response);
  if (responses.length !== 2) return;

  const statuses = responses.map(item => item.code).sort();
  console.log("Concurrent completion statuses:", statuses);

  pm.test("Timing-dependent split is reproduced", function () {
    pm.expect(statuses).to.eql([200, 409]);
  });

  const success = responses.find(item => item.code === 200);
  if (success) {
    pm.environment.set("observed_profile_revision", success.json().revision);
  }
}

pm.sendRequest(completion, collect);
pm.sendRequest(completion, collect);
```

Send it while session A still owns the lock. The two calls should remain pending.
A sequential Collection Runner is not acceptable evidence of concurrency.

### 6. Inspect blocking and lock ownership — 3 minutes

In pgAdmin session B, run:

```sql
SELECT
    pid,
    application_name,
    state,
    wait_event_type,
    wait_event,
    pg_blocking_pids(pid) AS blocked_by,
    LEFT(query, 140) AS query
FROM pg_stat_activity
WHERE datname = current_database()
  AND pid <> pg_backend_pid()
  AND (cardinality(pg_blocking_pids(pid)) > 0 OR query ILIKE '%onboarding_drafts%')
ORDER BY query_start;
```

Then inspect the relevant locks:

```sql
SELECT
    a.pid,
    l.locktype,
    l.mode,
    l.granted,
    l.relation::regclass AS relation,
    pg_blocking_pids(a.pid) AS blocked_by
FROM pg_stat_activity AS a
JOIN pg_locks AS l ON l.pid = a.pid
WHERE a.datname = current_database()
  AND (l.relation = 'onboarding_drafts'::regclass OR NOT l.granted)
ORDER BY a.pid, l.granted, l.mode;
```

Capture the blocker PID, waiting PIDs, wait event, lock mode, and timestamp in the
diagnosis document. Explain that PIDs are transient evidence, not configuration.

### 7. Release the lock and observe the defect — 2 minutes

In pgAdmin session A:

```sql
COMMIT;
```

Return to Postman. The assertion must pass with:

```text
[200, 409]
```

In pgAdmin verify the durable result:

```sql
SELECT revision, current_group, completed_at
FROM onboarding_drafts
WHERE user_id = '<TEST_USER_UUID>'::uuid;

SELECT profile_revision, COUNT(*)
FROM fitness_profile_snapshots
WHERE user_id = '<TEST_USER_UUID>'::uuid
GROUP BY profile_revision
ORDER BY profile_revision DESC;
```

The draft advances once and one snapshot exists for the successful profile
revision. This isolates the defect to retry semantics, not duplicate persistence.

### 8. Confirm the root cause in Cursor — 2.5 minutes

Return to `FitnessProfileRepository::complete` and show:

```rust
draft::Entity::find_by_id(user_id)
    .filter(draft::Column::Revision.eq(expected_draft_revision))
    .lock_exclusive()
```

Explain the observed `READ COMMITTED` timeline:

```text
request A waits -> locks revision R -> commits revision R+1
request B waits -> resumes -> revision predicate is rechecked -> no matching row
request B maps the missing match to RevisionConflict -> HTTP 409
```

Contrast this with a later retry: the service reads the already-completed draft
and can return 200 through its fast path. The same logical retry therefore changes
behavior solely because of timing.

### 9. Finalize diagnostic outputs and epilogue — 2 minutes

Export the Postman collection after clearing token values. Save the SQL above as:

```text
rust/diagnostics/onboarding-completion-locks.sql
```

Complete each `Answer:` section in
`rust/diagnosis/concurrency-diagnosis.md` using only evidence observed during the
recording. Keep the prompts unchanged so the blank input and completed output are
directly comparable.

Epilogue:

> I reproduced the failure deterministically, observed both API transactions
> waiting on the same PostgreSQL row, and proved that persistence remains atomic.
> The defect is the interaction between the revision predicate and post-wait row
> state. The next chain task can now implement against explicit, measurable
> acceptance criteria.

## Timing

| Phase | Time |
| --- | ---: |
| Opening and baseline | 1.5 min |
| Code-path inspection | 2.5 min |
| Reset test state | 2 min |
| Hold coordination row | 2 min |
| Concurrent Postman probe | 2.5 min |
| Lock inspection | 3 min |
| Release and persistence proof | 2 min |
| Root-cause confirmation | 2.5 min |
| Outputs and epilogue | 2 min |
| **Total** | **20 min** |

Do not modify production Rust logic in this task. Do not release the pgAdmin lock
until the waiting sessions have been visibly identified. Keep Cursor as the main
app and typing below 35%, with no uninterrupted typing segment over 45 seconds.
