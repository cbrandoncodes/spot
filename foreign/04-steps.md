# Task 04 — Idempotent Workout Set Execution

## Timebox and outcome

**Target: 28–30 minutes. Do not exceed 30 minutes.**

Protect workout history from mobile retries and weak gym connectivity:

```text
first command, key K              -> 200 and one performed set
same key K, identical payload     -> 200 and the same response
same key K, different payload     -> 409 idempotency_key_reused
two concurrent identical commands -> one durable performed set
```

The guarantee is one durable set effect per user-scoped idempotency key—not
general exactly-once network delivery.

## Scope

Implement only active workout sessions and performed-set recording. Do not add
program generation, exercise catalogs, progression, buddy workouts, or session
completion.

Expected files:

```text
rust/src/migration/m20261126_000001_workout_execution.rs
rust/src/migration/mod.rs
rust/src/modules/workout_execution/mod.rs
rust/src/modules/workout_execution/domain.rs
rust/src/modules/workout_execution/service.rs
rust/src/modules/workout_execution/http.rs
rust/src/modules/workout_execution/infrastructure/mod.rs
rust/src/modules/workout_execution/infrastructure/postgres/mod.rs
rust/src/modules/workout_execution/infrastructure/postgres/repository.rs
rust/src/modules/mod.rs
rust/src/lib.rs
rust/src/bin/api.rs
rust/src/bootstrap/http.rs
rust/docs/workout-execution.md
```

Use deterministic editor snippets for module boilerplate if desired; AI code
generation remains disabled. The engineering work to show is the constraint,
transaction, conflict semantics, and verification.

## Recording plan

### 1. Opening and failure model — 1.5 minutes

Show the approved Goal and explain:

> A client can lose the response after a performed set is committed. Retrying
> must return the original result without adding another set, while reuse of the
> same key for different work must be rejected.

Acceptance criteria:

1. The key is scoped to the authenticated user.
2. Session ownership and active status are checked transactionally.
3. Identical retries return the same response body and status.
4. Mismatched key reuse returns `409` without mutation.
5. Concurrent duplicates leave one row.

### 2. Design the invariant — 2 minutes

State the selected design:

- PostgreSQL unique constraint on `(user_id, idempotency_key)` is authoritative.
- The request payload is represented by stored set columns and compared on replay.
- `INSERT ... ON CONFLICT DO NOTHING RETURNING` avoids poisoning the transaction.
- A following `SELECT` reads the winner after a concurrent insert commits.
- The session row is locked so session closure cannot race set recording.

Alternatives in one line: a process cache fails across instances, check-then-insert
races, and an opaque response cache duplicates data already stored in the set row.

### 3. Add the database invariant — 4 minutes

Create `m20261126_000001_workout_execution.rs`:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"
                CREATE TABLE workout_sessions (
                    id UUID PRIMARY KEY,
                    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
                    status TEXT NOT NULL DEFAULT 'active',
                    started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    completed_at TIMESTAMPTZ NULL,
                    CONSTRAINT workout_sessions_owner_key UNIQUE (id, user_id),
                    CONSTRAINT workout_sessions_status_values
                        CHECK (status IN ('active', 'completed', 'cancelled')),
                    CONSTRAINT workout_sessions_completion_state CHECK (
                        (status = 'completed') = (completed_at IS NOT NULL)
                    )
                )
            "#,
            r#"
                CREATE INDEX workout_sessions_user_started_idx
                ON workout_sessions (user_id, started_at DESC)
            "#,
            r#"
                CREATE TABLE performed_sets (
                    id UUID PRIMARY KEY,
                    session_id UUID NOT NULL,
                    user_id UUID NOT NULL,
                    idempotency_key UUID NOT NULL,
                    exercise_code TEXT NOT NULL,
                    set_index SMALLINT NOT NULL,
                    reps SMALLINT NOT NULL,
                    load_grams INTEGER NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT performed_sets_session_owner_fk
                        FOREIGN KEY (session_id, user_id)
                        REFERENCES workout_sessions (id, user_id)
                        ON DELETE CASCADE,
                    CONSTRAINT performed_sets_idempotency_unique
                        UNIQUE (user_id, idempotency_key),
                    CONSTRAINT performed_sets_exercise_format CHECK (
                        exercise_code ~ '^[a-z0-9_]{1,64}$'
                    ),
                    CONSTRAINT performed_sets_index_range CHECK (set_index BETWEEN 1 AND 100),
                    CONSTRAINT performed_sets_reps_range CHECK (reps BETWEEN 1 AND 100),
                    CONSTRAINT performed_sets_load_range CHECK (
                        load_grams BETWEEN 0 AND 1000000
                    )
                )
            "#,
            r#"
                CREATE INDEX performed_sets_session_created_idx
                ON performed_sets (session_id, created_at, id)
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();
        for statement in [
            "DROP TABLE IF EXISTS performed_sets",
            "DROP TABLE IF EXISTS workout_sessions",
        ] {
            database.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
```

Register it last in `migration/mod.rs`:

```rust
mod m20261126_000001_workout_execution;
```

```rust
Box::new(m20261126_000001_workout_execution::Migration),
```

Run:

```bash
cd rust
cargo fmt
cargo check --bin migrate
cargo run --bin migrate
```

In pgAdmin, show both constraints on `performed_sets`. Reason: application checks
improve errors, but only the unique constraint coordinates every API instance.

### 4. Add domain types — 3 minutes

Create `modules/workout_execution/domain.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RecordSetCommand {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub idempotency_key: Uuid,
    pub exercise_code: String,
    pub set_index: i16,
    pub reps: i16,
    pub load_grams: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkoutSession {
    pub id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformedSet {
    pub id: Uuid,
    pub session_id: Uuid,
    pub idempotency_key: Uuid,
    pub exercise_code: String,
    pub set_index: i16,
    pub reps: i16,
    pub load_grams: i32,
    pub created_at: DateTime<Utc>,
}

impl PerformedSet {
    pub fn matches(&self, command: &RecordSetCommand) -> bool {
        self.session_id == command.session_id
            && self.idempotency_key == command.idempotency_key
            && self.exercise_code == command.exercise_code
            && self.set_index == command.set_index
            && self.reps == command.reps
            && self.load_grams == command.load_grams
    }
}
```

Create the module shells:

```rust
// modules/workout_execution/mod.rs
mod domain;
pub mod http;
mod infrastructure;
mod service;

pub use service::WorkoutExecutionService;
```

```rust
// modules/workout_execution/infrastructure/mod.rs
pub mod postgres;
```

```rust
// modules/workout_execution/infrastructure/postgres/mod.rs
mod repository;

pub use repository::{RepositoryError, WorkoutExecutionRepository};
```

### 5. Implement the atomic repository — 6 minutes

Create `infrastructure/postgres/repository.rs`:

```rust
use chrono::{DateTime, Utc};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement,
    TransactionTrait,
};
use thiserror::Error;
use uuid::Uuid;

use crate::modules::workout_execution::domain::{
    PerformedSet, RecordSetCommand, WorkoutSession,
};

#[derive(Clone)]
pub struct WorkoutExecutionRepository {
    database: DatabaseConnection,
}

impl WorkoutExecutionRepository {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    pub async fn start_session(
        &self,
        user_id: Uuid,
    ) -> Result<WorkoutSession, RepositoryError> {
        let id = Uuid::now_v7();
        let row = self
            .database
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                    INSERT INTO workout_sessions (id, user_id)
                    VALUES ($1, $2)
                    RETURNING id, status, started_at
                "#,
                [id.into(), user_id.into()],
            ))
            .await?
            .ok_or(RepositoryError::MissingResult)?;

        session_from_row(&row)
    }

    pub async fn record_set(
        &self,
        command: &RecordSetCommand,
    ) -> Result<PerformedSet, RepositoryError> {
        let transaction = self.database.begin().await?;

        let session = transaction
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                    SELECT status
                    FROM workout_sessions
                    WHERE id = $1 AND user_id = $2
                    FOR UPDATE
                "#,
                [command.session_id.into(), command.user_id.into()],
            ))
            .await?
            .ok_or(RepositoryError::SessionNotFound)?;

        let status: String = session.try_get("", "status")?;
        if status != "active" {
            return Err(RepositoryError::SessionNotActive);
        }

        let id = Uuid::now_v7();
        let inserted = transaction
            .query_one_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
                    INSERT INTO performed_sets (
                        id, session_id, user_id, idempotency_key,
                        exercise_code, set_index, reps, load_grams
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                    ON CONFLICT (user_id, idempotency_key) DO NOTHING
                    RETURNING id, session_id, idempotency_key, exercise_code,
                              set_index, reps, load_grams, created_at
                "#,
                [
                    id.into(),
                    command.session_id.into(),
                    command.user_id.into(),
                    command.idempotency_key.into(),
                    command.exercise_code.clone().into(),
                    command.set_index.into(),
                    command.reps.into(),
                    command.load_grams.into(),
                ],
            ))
            .await?;

        let row = match inserted {
            Some(row) => row,
            None => transaction
                .query_one_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"
                        SELECT id, session_id, idempotency_key, exercise_code,
                               set_index, reps, load_grams, created_at
                        FROM performed_sets
                        WHERE user_id = $1 AND idempotency_key = $2
                    "#,
                    [command.user_id.into(), command.idempotency_key.into()],
                ))
                .await?
                .ok_or(RepositoryError::MissingResult)?,
        };

        let recorded = set_from_row(&row)?;
        if !recorded.matches(command) {
            transaction.rollback().await?;
            return Err(RepositoryError::IdempotencyConflict);
        }

        transaction.commit().await?;
        Ok(recorded)
    }
}

fn session_from_row(row: &QueryResult) -> Result<WorkoutSession, RepositoryError> {
    Ok(WorkoutSession {
        id: row.try_get("", "id")?,
        status: row.try_get("", "status")?,
        started_at: row.try_get::<DateTime<Utc>>("", "started_at")?,
    })
}

fn set_from_row(row: &QueryResult) -> Result<PerformedSet, RepositoryError> {
    Ok(PerformedSet {
        id: row.try_get("", "id")?,
        session_id: row.try_get("", "session_id")?,
        idempotency_key: row.try_get("", "idempotency_key")?,
        exercise_code: row.try_get("", "exercise_code")?,
        set_index: row.try_get("", "set_index")?,
        reps: row.try_get("", "reps")?,
        load_grams: row.try_get("", "load_grams")?,
        created_at: row.try_get::<DateTime<Utc>>("", "created_at")?,
    })
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("workout session was not found")]
    SessionNotFound,
    #[error("workout session is not active")]
    SessionNotActive,
    #[error("idempotency key was already used for different set data")]
    IdempotencyConflict,
    #[error("database statement did not return its expected row")]
    MissingResult,
    #[error(transparent)]
    Database(#[from] DbErr),
}
```

Reason: `DO NOTHING` makes the losing insert wait for the unique-index winner
without aborting its transaction. The next statement sees the committed row and
compares its business payload before returning it.

### 6. Add service and HTTP contracts — 5 minutes

Create `service.rs`:

```rust
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use super::{
    domain::{PerformedSet, RecordSetCommand, WorkoutSession},
    infrastructure::postgres::{RepositoryError, WorkoutExecutionRepository},
};

#[derive(Clone)]
pub struct WorkoutExecutionService {
    repository: WorkoutExecutionRepository,
}

impl WorkoutExecutionService {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            repository: WorkoutExecutionRepository::new(database),
        }
    }

    pub async fn start_session(
        &self,
        user_id: Uuid,
    ) -> Result<WorkoutSession, RepositoryError> {
        self.repository.start_session(user_id).await
    }

    pub async fn record_set(
        &self,
        command: RecordSetCommand,
    ) -> Result<PerformedSet, RepositoryError> {
        self.repository.record_set(&command).await
    }
}
```

Create `http.rs`:

```rust
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use garde::Validate as _;
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, modules::auth::extractor::AuthenticatedUser};

use super::{
    domain::RecordSetCommand,
    infrastructure::postgres::RepositoryError,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/workout-sessions", post(start_session))
        .route("/api/workout-sessions/{session_id}/sets", post(record_set))
}

async fn start_session(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let session = state.workouts.start_session(identity.user_id).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecordSetRequest {
    #[garde(skip)]
    idempotency_key: Uuid,
    #[garde(length(bytes, min = 1, max = 64))]
    exercise_code: String,
    #[garde(range(min = 1, max = 100))]
    set_index: i16,
    #[garde(range(min = 1, max = 100))]
    reps: i16,
    #[garde(range(min = 0, max = 1_000_000))]
    load_grams: i32,
}

async fn record_set(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Path(session_id): Path<Uuid>,
    Json(request): Json<RecordSetRequest>,
) -> Result<Json<super::domain::PerformedSet>, ApiError> {
    request.validate().map_err(ApiError::validation)?;
    if !is_exercise_code(&request.exercise_code) {
        return Err(ApiError::invalid_exercise_code());
    }
    let command = RecordSetCommand {
        user_id: identity.user_id,
        session_id,
        idempotency_key: request.idempotency_key,
        exercise_code: request.exercise_code,
        set_index: request.set_index,
        reps: request.reps,
        load_grams: request.load_grams,
    };
    Ok(Json(state.workouts.record_set(command).await?))
}

fn is_exercise_code(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    detail: String,
}

impl ApiError {
    fn validation(error: garde::Report) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "invalid_performed_set",
            detail: error.to_string(),
        }
    }

    fn invalid_exercise_code() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "invalid_performed_set",
            detail: "exerciseCode must use lowercase letters, digits, or underscores".to_owned(),
        }
    }
}

impl From<RepositoryError> for ApiError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::SessionNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "workout_session_not_found",
                detail: error.to_string(),
            },
            RepositoryError::SessionNotActive => Self {
                status: StatusCode::CONFLICT,
                code: "workout_session_not_active",
                detail: error.to_string(),
            },
            RepositoryError::IdempotencyConflict => Self {
                status: StatusCode::CONFLICT,
                code: "idempotency_key_reused",
                detail: error.to_string(),
            },
            RepositoryError::MissingResult | RepositoryError::Database(_) => {
                tracing::error!(%error, "workout execution persistence failed");
                Self {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "workout_execution_unavailable",
                    detail: "Workout execution is temporarily unavailable.".to_owned(),
                }
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "type": format!("https://spot.app/problems/{}", self.code),
                "title": self.status.canonical_reason().unwrap_or("Request failed"),
                "status": self.status.as_u16(),
                "detail": self.detail,
                "code": self.code
            })),
        )
            .into_response()
    }
}
```

### 7. Wire, compile, and add focused tests — 2.5 minutes

Add to `modules/mod.rs`:

```rust
pub mod workout_execution;
```

Add the service import and field to `AppState` in `lib.rs`:

```rust
use crate::modules::workout_execution::WorkoutExecutionService;
```

```rust
pub workouts: WorkoutExecutionService,
```

In `bin/api.rs`:

```rust
let workouts =
    spot_backend::modules::workout_execution::WorkoutExecutionService::new(pool.clone());
```

Add `workouts` to `AppState`, and merge the routes in `bootstrap/http.rs`:

```rust
.merge(modules::workout_execution::http::routes())
```

Append these focused tests to `domain.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> RecordSetCommand {
        RecordSetCommand {
            user_id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            idempotency_key: Uuid::now_v7(),
            exercise_code: "bench_press".to_owned(),
            set_index: 1,
            reps: 8,
            load_grams: 60_000,
        }
    }

    fn recorded(command: &RecordSetCommand) -> PerformedSet {
        PerformedSet {
            id: Uuid::now_v7(),
            session_id: command.session_id,
            idempotency_key: command.idempotency_key,
            exercise_code: command.exercise_code.clone(),
            set_index: command.set_index,
            reps: command.reps,
            load_grams: command.load_grams,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn identical_payload_matches_the_recorded_set() {
        let command = command();
        assert!(recorded(&command).matches(&command));
    }

    #[test]
    fn changed_payload_does_not_match_the_recorded_set() {
        let original = command();
        let existing = recorded(&original);
        let mut changed = original.clone();
        changed.reps += 1;

        assert!(!existing.matches(&changed));
    }
}
```

Run:

```bash
cd rust
cargo fmt
cargo check --bin api
cargo test workout_execution
cargo clippy --all-targets --all-features -- -D warnings
```

Reason: pure comparison tests document when replay is safe; API-level concurrency
verification remains necessary because unit tests cannot prove unique-index waits.

### 8. Prove concurrent idempotency — 4 minutes

In Postman, start a session:

```http
POST {{base_url}}/api/workout-sessions
Authorization: Bearer {{spot_token}}
```

Save `body.id` as `workout_session_id`. Generate and save one UUID as
`set_idempotency_key`.

Create a request whose post-response script fires two concurrent calls:

```javascript
const request = {
  url: `${pm.environment.get("base_url")}/api/workout-sessions/` +
       `${pm.environment.get("workout_session_id")}/sets`,
  method: "POST",
  header: {
    Authorization: `Bearer ${pm.environment.get("spot_token")}`,
    "Content-Type": "application/json"
  },
  body: {
    mode: "raw",
    raw: JSON.stringify({
      idempotencyKey: pm.environment.get("set_idempotency_key"),
      exerciseCode: "bench_press",
      setIndex: 1,
      reps: 8,
      loadGrams: 60000
    })
  }
};

const responses = [];
function collect(error, response) {
  pm.expect(error).to.equal(null);
  responses.push(response);
  if (responses.length !== 2) return;

  pm.test("Concurrent retries both succeed", function () {
    pm.expect(responses.map(item => item.code).sort()).to.eql([200, 200]);
  });
  pm.test("Concurrent retries return one set", function () {
    const ids = responses.map(item => item.json().id);
    pm.expect(new Set(ids).size).to.equal(1);
  });
}

pm.sendRequest(request, collect);
pm.sendRequest(request, collect);
```

Reuse the same key with `reps: 9`; assert:

```javascript
pm.test("Changed payload cannot reuse the key", function () {
  pm.response.to.have.status(409);
  pm.expect(pm.response.json().code).to.equal("idempotency_key_reused");
});
```

In pgAdmin:

```sql
SELECT id, session_id, idempotency_key, exercise_code, reps, load_grams
FROM performed_sets
WHERE user_id = '<TEST_USER_UUID>'::uuid
  AND idempotency_key = '<IDEMPOTENCY_UUID>'::uuid;
```

Expected: exactly one row containing the original eight-rep payload.

### 9. Document, review, and epilogue — 1.5 minutes

Document the endpoint contract and retry rules in `rust/docs/workout-execution.md`.
Run:

```bash
cd rust
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
git diff --stat
```

Review the diff in Cursor, then summarize:

- database uniqueness coordinates every API instance;
- identical retries return the stored result;
- changed payloads cannot steal a key;
- concurrency produced one durable performed set;
- Task 5 can now trust workout history as progression input.

State that the approved Goal is complete.

## Timing and compliance

| Phase | Time |
| --- | ---: |
| Opening | 1.5 min |
| Invariant and alternatives | 2 min |
| Migration and pgAdmin | 4 min |
| Domain and module structure | 3 min |
| Atomic repository | 6 min |
| Service and HTTP | 5 min |
| Wiring and focused tests | 2.5 min |
| Concurrent API/database proof | 4 min |
| Documentation and epilogue | 1.5 min |
| **Total** | **29.5 min** |

Cursor should remain the main application for at least 18 minutes. Use snippets
for deterministic boilerplate, keep typing below 35%, and pause within every
45-second typing window to navigate types, compile, inspect constraints, or explain
the transaction. If delayed, shorten build waiting—not the conflict or persistence
proof.
