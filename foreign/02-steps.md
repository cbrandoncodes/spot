# Task 02 — Deterministic Onboarding Completion Retries

## Timebox and outcome

**Target: 28–30 minutes. Do not exceed 30 minutes.**

Fix one production concurrency defect:

```text
Before: two concurrent completion requests at revision R -> [200, 409]
After:  the same logical completion command           -> [200, 200]
Guard:  a genuinely older command                     -> 409
Effect: one durable completion effect per accepted revision
```

Avoid claiming general “exactly-once request execution.” The narrower guarantee
is that retries for one accepted draft revision share one durable profile result.

## Scope

Expected files:

```text
rust/src/modules/fitness_profile/service.rs
rust/src/modules/fitness_profile/infrastructure/postgres/repository.rs
rust/docs/onboarding.md
```

Do not add a migration, idempotency table, process mutex, forced rollback test,
or unrelated onboarding changes.

## Prepare before recording

- Disable Cursor AI and any other coding assistants.
- Save the authenticated Postman concurrency request and its assertions.
- Have `base_url`, `spot_token`, `completion_revision`, and the test user UUID in
  the Postman environment.
- Use a completed test account whose movement-wellness group can reopen its draft.
- Confirm Task 1 is committed and all checks pass.

The concurrency request must use two `pm.sendRequest` calls, not the sequential
Collection Runner. Its tests should automatically assert:

- both status codes are `200`;
- both successful bodies have the same profile revision;
- a separate stale request returns `409`.

Create a Postman request named `Concurrent onboarding completion`. Its main
request can call `GET {{base_url}}/health/live`. Add this exact post-response
script so the two completion calls are genuinely concurrent:

```javascript
const completionRequest = {
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

  pm.test("Both completion retries succeed", function () {
    pm.expect(statuses).to.eql([200, 200]);
  });

  if (statuses.every(status => status === 200)) {
    const revisions = responses.map(item => item.json().revision);
    pm.test("Both calls return the same profile revision", function () {
      pm.expect(new Set(revisions).size).to.equal(1);
    });
  }
}

pm.sendRequest(completionRequest, collect);
pm.sendRequest(completionRequest, collect);
```

Before the fix, the first assertion intentionally fails because the observed
statuses should be `[200, 409]`. After the fix, do not alter the script; the same
test must pass as `[200, 200]`.

At recording start, leave applications empty or minimized. Locate and open the
approved Goal document, repository, Postman collection, and pgAdmin inputs on
camera. Do not show secrets.

## Recording plan

### 1. Opening and approved goal — 1.5 minutes

Show the exact approved Goal document and state:

> Make onboarding completion retries deterministic under concurrency while
> preserving optimistic concurrency for genuinely stale commands.

Present the four outcomes at the top of this guide. Explain that Cursor will be
the main application and Postman/pgAdmin will supply runtime evidence.

### 2. Reproduce the failure — 4 minutes

In Postman:

1. `GET /api/onboarding` and save the current revision.
2. `PATCH /api/onboarding/groups/movement-wellness` to reopen the completed draft.
3. Save the response revision as `completion_revision`.

Use this body for step 2:

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

Make the overlap deterministic without adding application instrumentation. In
pgAdmin session A, temporarily hold the draft row:

```sql
BEGIN;
SELECT user_id, revision
FROM onboarding_drafts
WHERE user_id = '<TEST_USER_UUID>'::uuid
FOR UPDATE;
```

Run the saved Postman request that fires two completion calls. Both should wait.
Do not wait for responses yet.

### 3. Inspect the lock and diagnose — 3 minutes

In pgAdmin session B, show the blocked API work:

```sql
SELECT pid, state, wait_event_type, wait_event,
       pg_blocking_pids(pid) AS blocked_by,
       LEFT(query, 100) AS query
FROM pg_stat_activity
WHERE datname = current_database()
  AND pid <> pg_backend_pid()
ORDER BY query_start;
```

Optionally open `pg_locks` graphically if it is already configured. Then run
`COMMIT;` in session A. Postman should report the failing baseline `[200, 409]`.

In Cursor, trace:

```text
complete_onboarding
FitnessProfileService::complete
FitnessProfileRepository::complete
```

Highlight that the repository combines the revision predicate with `FOR UPDATE`.
Under PostgreSQL `READ COMMITTED`, the waiting statement rechecks the updated row;
after the winner increments the revision, it no longer matches and becomes `409`.

### 4. Decide the design — 2 minutes

Narrate the decision before editing:

1. Lock the draft by stable identity (`user_id`).
2. Inspect its committed state only after acquiring the lock.
3. Proceed when the locked revision equals the submitted revision.
4. Return the existing result when a completed draft advanced exactly once.
5. Reject older or incompatible state as a real revision conflict.

Also address the read-after-commit concern:

- Remove the service-level completed-draft shortcut so the locked repository
  decision is authoritative.
- Read the existing profile through the active transaction before releasing the
  draft lock.
- Reuse a transaction-aware profile-loading helper rather than opening another
  pooled connection while a transaction is held.

This is safer than committing and then calling `find_profile()`, because another
valid state transition cannot overtake the retry before its result is read.

Alternatives in one line: a process mutex fails across instances, serializable
isolation adds broad retry complexity, and a new idempotency table duplicates the
command identity already provided by the draft revision.

### 5. Implement while reasoning — 8 minutes

The exact edits are below for preparation and correctness. During the recording,
make them in short sections and explain the invariant each section establishes.

#### 5.1 Make profile loading transaction-aware

In `repository.rs`, add `ConnectionTrait` to the SeaORM imports:

```rust
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait,
    DatabaseConnection, DbErr, EntityTrait, ExprTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::Expr,
};
```

Replace the body of the existing public `find_profile` method with:

```rust
pub async fn find_profile(
    &self,
    user_id: Uuid,
) -> Result<Option<FitnessProfileRecord>, RepositoryError> {
    find_profile_with_connection(&self.database, user_id).await
}
```

Move its original body into this private function immediately below the
repository `impl`:

```rust
async fn find_profile_with_connection<C>(
    database: &C,
    user_id: Uuid,
) -> Result<Option<FitnessProfileRecord>, RepositoryError>
where
    C: ConnectionTrait,
{
    // Move the complete original find_profile body here.
}
```

Inside the moved body make these exact mechanical replacements:

```text
.one(&self.database)  -> .one(database)
.all(&self.database)  -> .all(database)
```

Do not change the mapping or validation logic. This helper can now read through
either the pool or the active `DatabaseTransaction` without acquiring a second
pooled connection.

#### 5.2 Add the post-lock decision

Add this below `FitnessProfileRecord`:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompletionDecision {
    Proceed,
    ReturnExisting,
}

fn decide_completion(
    draft: &draft::Model,
    expected_revision: i64,
) -> Result<CompletionDecision, RepositoryError> {
    let is_complete = draft.current_group == OnboardingGroup::Complete.as_str()
        && draft.completed_at.is_some();
    let just_completed = expected_revision.checked_add(1) == Some(draft.revision);

    if is_complete && (draft.revision == expected_revision || just_completed) {
        return Ok(CompletionDecision::ReturnExisting);
    }

    if draft.revision == expected_revision {
        return Ok(CompletionDecision::Proceed);
    }

    Err(RepositoryError::RevisionConflict)
}
```

Reason: the pure function distinguishes a legitimate replay from an unrelated
stale command after the database lock has revealed the latest committed state.

#### 5.3 Lock first and decide second

At the start of `FitnessProfileRepository::complete`, replace the existing draft
lookup with:

```rust
let transaction = self.database.begin().await?;
let draft = draft::Entity::find_by_id(user_id)
    .lock_exclusive()
    .one(&transaction)
    .await?
    .ok_or(RepositoryError::MissingDraft)?;

match decide_completion(&draft, expected_draft_revision)? {
    CompletionDecision::Proceed => {}
    CompletionDecision::ReturnExisting => {
        let existing = find_profile_with_connection(&transaction, user_id)
            .await?
            .ok_or(RepositoryError::MissingProfile)?;
        transaction.commit().await?;
        return Ok(existing);
    }
}
```

The profile is read before `commit`, while the transaction still holds the draft
lock. Do not use the previous commit-then-`self.find_profile()` ordering.

Near the end of `complete`, replace the draft update and final return with:

```rust
let next_draft_revision = draft.revision + 1;
let mut active_draft = draft.into_active_model();
active_draft.current_group = Set(OnboardingGroup::Complete.as_str().to_owned());
active_draft.revision = Set(next_draft_revision);
active_draft.completed_at = Set(Some(now));
active_draft.updated_at = Set(now);
active_draft.update(&transaction).await?;

let persisted = find_profile_with_connection(&transaction, user_id)
    .await?
    .ok_or(RepositoryError::MissingProfile)?;
transaction.commit().await?;
Ok(persisted)
```

Reason: the next revision comes from the locked row, and both the normal and
idempotent paths read the exact durable result inside the transaction.

#### 5.4 Remove the unlocked service shortcut

In `service.rs`, replace only `FitnessProfileService::complete` with:

```rust
pub async fn complete(
    &self,
    user_id: Uuid,
    revision: i64,
) -> Result<FitnessProfileRecord, ServiceError> {
    let draft = self.get_onboarding(user_id).await?;
    let is_immediate_completion_retry = draft.completed_at.is_some()
        && revision.checked_add(1) == Some(draft.revision);
    if !is_immediate_completion_retry {
        ensure_revision(revision, draft.revision)?;
    }
    if draft.questionnaire_version != QUESTIONNAIRE_VERSION {
        return Err(ServiceError::QuestionnaireVersionUnsupported);
    }
    let profile = draft.answers.complete()?;
    self.repository
        .complete(user_id, revision, &profile)
        .await
        .map_err(Into::into)
}
```

The service permits only its observed current revision or the immediately
completed retry to reach the repository. The repository still makes the final,
authoritative decision after locking; this prevents a caller from pairing a
newer submitted revision with profile answers read from an older draft. Keep
`CompletedProfileMissing` unchanged for this focused task—removing a public error
variant and its HTTP mapping is unrelated cleanup.

Run a quick compiler check before adding tests:

```bash
cd rust
cargo fmt
cargo check --bin api
```

Keep individual typing bursts below 45 seconds. Use the pauses to navigate types,
explain the lock invariant, compile, and review the diff. Total actual typing must
remain below 35% of the recording.

### 6. Focused tests — 3 minutes

Append this test module to `repository.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn draft_at(revision: i64, complete: bool) -> draft::Model {
        let now = Utc::now();
        draft::Model {
            user_id: Uuid::now_v7(),
            questionnaire_version: QUESTIONNAIRE_VERSION,
            current_group: if complete {
                OnboardingGroup::Complete.as_str().to_owned()
            } else {
                OnboardingGroup::MovementWellness.as_str().to_owned()
            },
            answers: serde_json::json!({}),
            revision,
            completed_at: complete.then_some(now),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn current_incomplete_revision_can_proceed() {
        assert!(matches!(
            decide_completion(&draft_at(4, false), 4),
            Ok(CompletionDecision::Proceed)
        ));
    }

    #[test]
    fn just_completed_revision_returns_existing() {
        assert!(matches!(
            decide_completion(&draft_at(5, true), 4),
            Ok(CompletionDecision::ReturnExisting)
        ));
    }

    #[test]
    fn current_completed_revision_is_a_no_op() {
        assert!(matches!(
            decide_completion(&draft_at(5, true), 5),
            Ok(CompletionDecision::ReturnExisting)
        ));
    }

    #[test]
    fn genuinely_stale_revision_conflicts() {
        assert!(matches!(
            decide_completion(&draft_at(7, true), 4),
            Err(RepositoryError::RevisionConflict)
        ));
    }
}
```

Run:

```bash
cd rust
cargo fmt
cargo test modules::fitness_profile
```

Use the test output to confirm the decision boundary. Do not narrate routine
compiler output line by line.

### 7. API and database regression proof — 4 minutes

Reopen the draft again. Repeat the same pgAdmin row hold, run the same concurrent
Postman request, inspect the blocked sessions briefly, and release the lock.

Postman must now automatically show:

```text
statuses: [200, 200]
same profile revision: PASS
```

Verify the durable result in pgAdmin:

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

Show one draft increment and one snapshot for the returned profile revision.
Then create a direct stale request:

```http
POST {{base_url}}/api/onboarding/complete
Authorization: Bearer {{spot_token}}
Content-Type: application/json
```

Use a known revision at least two behind the current draft:

```json
{
  "revision": {{stale_completion_revision}}
}
```

Add these Postman assertions:

```javascript
pm.test("Genuinely stale completion is rejected", function () {
  pm.response.to.have.status(409);
});

pm.test("Conflict uses the onboarding revision code", function () {
  pm.expect(pm.response.json().code)
    .to.equal("onboarding_revision_conflict");
});
```

This proves idempotency did not disable optimistic concurrency.

### 8. Documentation, diff, and checks — 2 minutes

Document the contract briefly in `rust/docs/onboarding.md`:

```text
Concurrent completion requests for the same accepted draft revision return the
same profile result while producing one durable completion effect. Older draft
revisions still return onboarding_revision_conflict.
```

Run or show completed results for:

```bash
cd rust
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
git diff --stat
```

Review the real diff in Cursor. Explain why each changed file is necessary and
confirm that no migration or new infrastructure was introduced.

### 9. Epilogue — 1 minute

Clearly separate the epilogue from active work. Summarize:

- the original timing-dependent `[200, 409]` result;
- the post-lock state decision;
- automated `[200, 200]`, equal-revision, single-snapshot, and stale-`409` proof;
- the production guarantee: one durable completion effect per accepted revision.

State that the approved Goal is complete, then end the recording.

## Timing and compliance

| Phase | Time |
| --- | ---: |
| Opening | 1.5 min |
| Failure reproduction | 4 min |
| Lock inspection and diagnosis | 3 min |
| Design decision | 2 min |
| Implementation | 8 min |
| Focused tests | 3 min |
| Regression proof | 4 min |
| Documentation and checks | 2 min |
| Epilogue | 1 min |
| **Total** | **28.5 min** |

Cursor should remain visible for at least 16–17 minutes overall. Do not sacrifice
the diagnosis, automated assertions, database evidence, or epilogue to save time.
If delayed, shorten navigation and terminal waiting—not the engineering proof.
