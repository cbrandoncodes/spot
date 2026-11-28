# Task 02-02 — Implement Deterministic Onboarding Completion Retries

## Outcome and timebox

**Target: 26–29 minutes. Hard maximum: 30 minutes.**

Continue directly from Task 02-01. Consume its failing Postman collection, SQL
lock probes, and diagnosis to implement this behavior:

```text
same accepted completion command -> [200, 200], same profile result
durable database effect          -> one draft increment and one snapshot
genuinely older revision         -> 409 onboarding_revision_conflict
```

Describe the guarantee as one durable completion effect per accepted revision,
not general exactly-once request execution.

## Inputs from Task 02-01

```text
Spot-Onboarding-Concurrency-Reproducer.postman_collection.json
onboarding-completion-locks.sql
onboarding-concurrency-diagnosis.md
```

Production files to modify:

```text
rust/src/modules/fitness_profile/infrastructure/postgres/repository.rs
rust/src/modules/fitness_profile/service.rs
rust/docs/onboarding.md
```

Do not add a migration, idempotency table, process mutex, or new public endpoint.

## Recording topology

```text
Cursor LOOKUP -> Cursor MODIFY/TEST -> Postman MODIFY/CREATE
              -> Postman LOOKUP -> pgAdmin LOOKUP -> Cursor LOOKUP/DIFF
```

Cursor remains the main app for at least 17 minutes.

## Recording steps

### 1. Open the chain evidence — 1.5 minutes

Show Task 02-01’s diagnosis and failing `[200, 409]` collection result. State:

> I will change the transaction so every API instance decides from the same
> locked draft state, while retaining the existing stale-write contract.

Run the unchanged focused baseline:

```bash
cd rust
cargo test modules::fitness_profile
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. Make the design judgment — 2.5 minutes

Explain the selected transition:

1. Lock the draft by stable `user_id`, not `user_id + revision`.
2. Inspect the row after acquiring the lock.
3. Proceed if locked revision equals the command revision.
4. Return the existing result if a complete draft advanced exactly once.
5. Reject every older or incompatible state.

Also decide that profile loading must happen through the active transaction before
the draft lock is released.

Alternatives in one line: a process mutex fails across instances; serializable
isolation adds broad retry requirements; a new idempotency table duplicates the
natural command identity already supplied by the draft revision.

### 3. Make profile loading transaction-aware — 4 minutes

In `repository.rs`, add `ConnectionTrait`:

```rust
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait,
    DatabaseConnection, DbErr, EntityTrait, ExprTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    sea_query::Expr,
};
```

Replace the public lookup body with:

```rust
pub async fn find_profile(
    &self,
    user_id: Uuid,
) -> Result<Option<FitnessProfileRecord>, RepositoryError> {
    find_profile_with_connection(&self.database, user_id).await
}
```

Move its original body to:

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

Inside the moved body replace exactly:

```text
.one(&self.database) -> .one(database)
.all(&self.database) -> .all(database)
```

Reason: reuse one mapping implementation with either the pool or the active
transaction; do not acquire another pooled connection while holding the row lock.

### 4. Add the post-lock decision — 3 minutes

Add below `FitnessProfileRecord`:

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

Reason: only the current command and its immediately completed replay are safe;
idempotency must not convert arbitrary stale writes into success.

### 5. Lock first and decide second — 4 minutes

Replace the start of `FitnessProfileRepository::complete` with:

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

Replace the draft update and final return with:

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

Reason: derive the new revision from authoritative locked state and read the exact
result before another valid transition can overtake the retry.

### 6. Preserve a safe service gate — 2.5 minutes

Replace `FitnessProfileService::complete` with:

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

Reason: the service prevents pairing a newer submitted revision with answers read
from an older draft, while the repository remains authoritative after locking.

### 7. Add focused decision tests — 3 minutes

Append to `repository.rs`:

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
cargo check --bin api
cargo test modules::fitness_profile
cargo clippy --all-targets --all-features -- -D warnings
```

### 8. Convert the reproducer into a regression test — 3 minutes

Use `POST /api/onboarding/reopen` and save its returned revision. In the Task
02-01 Postman collection change only this assertion:

```javascript
pm.test("Concurrent retries both succeed", function () {
  pm.expect(statuses).to.eql([200, 200]);
});

const revisions = responses.map(item => item.json().revision);
pm.test("Concurrent retries return the same profile", function () {
  pm.expect(new Set(revisions).size).to.equal(1);
});
```

Repeat Task 02-01’s pgAdmin row hold and lock inspection, then `COMMIT`. The same
queued requests must now pass as `[200, 200]`.

Send a revision at least two behind and assert:

```javascript
pm.test("Genuinely stale completion is rejected", function () {
  pm.response.to.have.status(409);
  pm.expect(pm.response.json().code)
    .to.equal("onboarding_revision_conflict");
});
```

### 9. Verify persistence, document, and close — 2 minutes

Run Task 02-01’s database queries and prove:

- the draft advanced exactly once;
- both HTTP bodies report the same profile revision;
- exactly one snapshot exists for that profile revision.

Add to `rust/docs/onboarding.md`:

```markdown
Concurrent completion requests for the same accepted draft revision return the
same profile result while producing one durable completion effect. Older draft
revisions still return `409 onboarding_revision_conflict`.
```

Run:

```bash
git diff --check
git diff --stat
```

Epilogue:

> The failing chain artifact now passes without changing its concurrency method.
> Completion decisions come from the locked draft state, concurrent retries return
> one profile result, the database contains one durable effect, and genuinely stale
> commands still conflict. The diagnosis from Task 02-01 is fully resolved.

## Timing

| Phase | Time |
| --- | ---: |
| Chain evidence and baseline | 1.5 min |
| Design judgment | 2.5 min |
| Transaction-aware loading | 4 min |
| Completion decision | 3 min |
| Lock-first repository update | 4 min |
| Service gate | 2.5 min |
| Focused tests | 3 min |
| Postman regression proof | 3 min |
| Persistence, docs, epilogue | 2 min |
| **Total** | **25.5 min** |

Keep Cursor as the main app, typing below 35%, and each typing burst below 45
seconds. Do not change the Task 02-01 concurrency mechanism after the fix; using
the identical experiment is what makes the before/after evidence credible.
