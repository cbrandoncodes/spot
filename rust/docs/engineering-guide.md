# Spot Backend Engineering Guide

This document is the default technical direction for the Spot backend. It is a
living decision record: update it deliberately when a decision changes, and
explain the reason in the same pull request.

## Product and engineering principles

Spot's core promise is **Never feel lost at the gym again.** Guided workout
execution is the center of the product. Social functionality exists to help
people train, not to turn Spot into a general-purpose social network.

Engineering decisions should optimize for:

1. Correct workout and progression history.
2. Simple, explicit code over hidden framework behavior.
3. Strong privacy boundaries, particularly for health-adjacent and location data.
4. A modular monolith that can be split only when production evidence justifies it.
5. Operational simplicity for a small team.

## Architecture: a modular monolith

Begin with one Rust workspace and two runtime entry points:

- `api`: HTTP and WebSocket traffic.
- `worker`: transactional-outbox events, notifications, and background jobs.

They share domain crates/modules and a PostgreSQL database. Do not create
microservices for individual domains initially. Module boundaries and table
ownership are still mandatory.

Recommended source layout:

```text
rust/
├── Cargo.toml
├── migrations/
├── queries/                    # cross-domain/admin read models only
├── src/
│   ├── lib.rs
│   ├── bin/
│   │   ├── api.rs
│   │   └── worker.rs
│   ├── bootstrap/
│   │   ├── config.rs
│   │   ├── database.rs
│   │   ├── http.rs
│   │   └── telemetry.rs
│   ├── shared/                 # deliberately small; never a dumping ground
│   │   ├── error.rs
│   │   ├── id.rs
│   │   ├── pagination.rs
│   │   └── time.rs
│   └── modules/
│       └── <domain>/
│           ├── mod.rs          # the module's public API
│           ├── domain/         # entities, value objects, rules, domain errors
│           ├── application/    # use cases and transaction boundaries
│           ├── infrastructure/
│           │   └── postgres/   # repositories and module-owned SQL
│           │       └── queries/
│           └── http/           # routes, request/response DTOs, error mapping
└── tests/
    ├── api/
    └── support/
```

Use modules inside the initial crate. Extract a module into its own crate only
when doing so creates a real enforcement or build-time benefit.

### Dependency direction

```text
HTTP adapter -> application use case -> domain
                         |
                         v
               infrastructure ports
                         |
                         v
                  PostgreSQL/external APIs
```

- Domain code must not import Axum, SQLx, HTTP DTOs, or vendor SDKs.
- HTTP handlers authenticate, validate, call one application use case, and map
  its result. They do not contain business rules or SQL.
- Application use cases coordinate domain behavior, repositories, transactions,
  and emitted events.
- Infrastructure implements persistence and external-service boundaries.
- Define a trait only at a meaningful boundary. Do not create a generic
  repository abstraction or an interface for every struct.
- A domain may use another domain only through the other domain's public
  application API or a domain event. It must not mutate another module's tables.

## Non-negotiable code organization rules

### SQL ownership

Executable application SQL may exist only in:

- `migrations/` for schema changes;
- `modules/<domain>/infrastructure/postgres/` for domain-owned persistence; or
- root `queries/` for explicitly named cross-domain read models such as admin
  reporting.

No SQL is allowed in handlers, application services, domain code, generic
helpers, or tests other than narrowly scoped test setup.

Prefer `sqlx::query_file!` / `query_file_as!` for substantial queries. A short,
single-purpose query may be inline only inside the owning Postgres repository.
Query file names describe intent, for example `find_active_program_for_user.sql`.

Each table has one owning domain. Cross-domain foreign keys are allowed when
they protect a genuine invariant, but write access still belongs to the owner.
Cross-domain joins belong in purpose-built read models, not accidental repository
coupling.

### No generic `utils`

Do not add `utils.rs`, `helpers.rs`, `common.rs`, or a miscellaneous dumping
ground. Put behavior beside the concept that owns it. If code is genuinely
shared, give it a precise name and keep its API narrow under `shared/`.

### Models and DTOs

Do not use one struct for every layer.

- HTTP request/response DTOs describe the public API.
- Domain types encode business meaning and invariants.
- Database row types represent query results.

Conversions between them should be explicit. Prevent impossible states with
value objects where useful, but avoid ceremonial wrappers with no invariant.

### Errors

- Use `thiserror` for typed domain, application, and infrastructure errors.
- Use `anyhow` only at executable/bootstrap boundaries and in one-off operational
  tooling where callers do not need to branch on the error.
- Map errors to one consistent RFC 9457-style JSON problem response.
- Never expose database errors, secrets, or internal implementation details to clients.

### Transactions and events

The application use case owns the transaction boundary. A repository must be
able to participate in that transaction instead of silently opening its own.

When a successful database change must cause an external effect, insert a
versioned event into a transactional `outbox_events` table in the same
transaction. The worker processes it with retries and idempotency. Do not publish
an event or push notification before the database commit succeeds.

## Initial technology choices

| Concern | Initial choice | Direction |
|---|---|---|
| Language | Stable Rust, edition 2024 | Pin the toolchain in `rust-toolchain.toml`. |
| Service shape | Modular monolith | API and worker binaries; no early microservices. |
| Async runtime | Tokio | Restrict features to those actually used once the base is stable. |
| HTTP API | Axum + Tower middleware | REST/JSON under `/v1`; thin handlers and nested domain routers. |
| Live transport | Axum WebSockets | Buddy workouts and chat need bidirectional traffic. |
| Database | PostgreSQL | System of record for all durable product data. |
| Database access | SQLx | Explicit SQL, typed mapping, pools, migrations, and compile-time checking. |
| IDs | UUIDv7 generated in Rust | Time-sortable, client-independent identifiers; never expose sequential IDs. |
| Date/time | `time` crate; `timestamptz` in Postgres | Store instants in UTC; localize only at presentation/scheduling edges. |
| Serialization | Serde / `serde_json` | JSON at API and event boundaries; typed columns for queryable data. |
| Validation | `garde` or small domain constructors | Transport shape validation first; business validation in the domain. |
| API contract | OpenAPI generated with `utoipa` | Commit the generated contract and check breaking changes in CI. |
| Configuration | Typed `Settings` loaded from environment | Fail fast at startup; secrets never get defaults or enter logs. |
| Secrets in memory | `secrecy` | Reduce accidental debug/log exposure. |
| Password hashing | Argon2id via `argon2` | Use per-password salts and versioned parameters. |
| Google/Apple sign-in | OpenID Connect/OAuth libraries behind an identity provider port | Verify issuer, audience, nonce, and provider keys server-side. |
| Access sessions | Short-lived access tokens plus rotated opaque refresh sessions | Store only hashed refresh secrets; support per-device revocation. |
| Observability | `tracing` + `tracing-subscriber`, then OpenTelemetry exporter | Structured spans with request/job IDs; never log tokens or sensitive fitness data. |
| Object storage | S3-compatible storage via presigned operations | Store object keys and metadata in Postgres, never large media blobs. |
| Background work | PostgreSQL job/outbox tables with `FOR UPDATE SKIP LOCKED` | Add a dedicated broker only after measured throughput demands it. |
| Caching | None initially | Add Redis only for demonstrated cache, coordination, or fan-out needs. |
| Search | PostgreSQL full-text search + `pg_trgm` | Enough for usernames, gyms, exercises, and programs initially. |
| Geospatial | PostgreSQL `cube` + `earthdistance` initially | The current Railway instance does not offer PostGIS. Keep coordinates behind the gym repository so PostGIS remains a future option. Never expose continuous user location by default. |
| Metrics | Prometheus-compatible metrics | HTTP latency, DB pool, jobs, WebSockets, and product-critical counters. |
| Testing | Rust unit tests + `sqlx::test` integration tests | Real PostgreSQL for persistence tests; avoid mocking SQL behavior. |
| Lint/format | rustfmt + Clippy with warnings denied in CI | Exceptions are local, documented, and rare. |

Crate versions must be selected and locked when implementation begins. Avoid
adding a dependency for functionality that is clearer in a small local type or
function. New production dependencies require a brief rationale and a check of
maintenance, license, and transitive scope.

## Domain boundaries and technical direction

### Identity

Owns credentials, external identities, sessions, email verification, account
state, and account deletion workflow. It does not own the public user profile.
Use Argon2id for passwords and standards-based OIDC/OAuth for Google and Apple.
Rate-limit sign-in, code delivery, code verification, and account recovery.

### Profile

Owns username, name, avatar reference, bio, public/private state, discoverability,
and social privacy preferences. Store a separately normalized username for
uniqueness and search. Do not use locale-sensitive application lowercasing as a
uniqueness guarantee.

### Fitness profile and onboarding

Owns onboarding answers and produces a versioned, normalized recommendation
input. Keep raw answers when useful for editing/audit, but recommendations consume
the normalized model rather than transport DTOs. Limitations and injuries are
sensitive and private by default.

### Gyms and location

Owns gyms, branches, equipment inventory, user-to-gym associations, and gym
verification. Use PostgreSQL `cube` + `earthdistance` for indexed MVP proximity
queries because PostGIS is unavailable on the current Railway instance. Keep the
coordinate representation behind this module's repository so a future PostGIS
migration does not leak across the codebase. Community membership should be based
on explicit gym association, not background tracking. District is a coarse
fallback; exact user location is not social data.

### Exercise content

Owns the canonical exercise taxonomy, muscles, patterns, equipment,
contraindications, substitutions, progressions, cues, and versioned media
metadata. PostgreSQL is the metadata source of truth; media lives in object
storage/CDN. Publishing is an explicit reviewed state transition.

### Programs

Owns curated program definitions, versions, phases, weeks, workout templates,
targets, alternatives, and progression rules. A published program version is
immutable. User enrollment points to a precise version so later editorial edits
cannot rewrite workout history.

### Workout execution

Owns workout sessions, exercise instances, performed sets, timers, substitutions,
feedback, pain flags, pause/resume, and completion. This is the most
transaction-sensitive domain. Commands such as start session, record set, and
finish session require idempotency keys or equivalent duplicate protection.
Use optimistic concurrency/version fields for updates that can arrive from
multiple devices or a buddy session.

### Progression and recommendations

Start as a deterministic Rust rules engine. Inputs and the selected rule-set
version must be recorded with each recommendation so results are explainable and
reproducible. It reads workout/fitness facts through explicit ports and emits
recommendations; it does not rewrite historical sessions. Add statistical or ML
components only behind the same interface once data and evaluation exist.

### Progress tracking

PRs, volume, streaks, and consistency are derived projections from immutable
workout facts where possible. Maintain projections asynchronously if computation
becomes expensive, but make rebuilding them possible. Measurements and progress
photos have stricter visibility than normal profile data.

### Friend graph

Owns friend requests, friendships, blocks, mutes, and discovery policy. Enforce a
canonical ordering for friendship pairs to prevent duplicates. Blocking must be
checked by messaging, activity, community, buddy workouts, and search—not
reimplemented differently by each domain.

### Activity feed

Owns posts, workout activity cards, captions, reactions, comments, and visibility.
It consumes workout events but never becomes the source of truth for workout
history. Build a fan-out-on-read feed first; reconsider only after measuring scale.
Use PostgreSQL keyset pagination, not offsets, for timelines.

### Buddy workouts

Owns invitations and shared-session coordination while each participant's durable
performance remains in workout execution. PostgreSQL stores authoritative state;
WebSockets deliver live deltas. Messages carry event IDs and sequence/version
information so reconnecting clients can recover rather than assume delivery.

### Accountability

Owns the explicit relationship and fine-grained consent grants. Every disclosure
is checked against the current grant. It consumes events such as planned,
started, completed, or missed; it must not expose wellness, injury, weight, or
body data through broad permissions.

### Messaging

Owns conversations, membership, messages, receipts, reactions, replies, and
attachments. Persist before broadcasting. WebSockets are delivery acceleration,
not durable storage. Use cursor-based history and enforce block/messaging policy
on every send. Add Redis or a broker for cross-instance fan-out only when running
multiple API instances requires it.

### Communities

Owns gym/district community membership, challenges, roles, and community content.
Gym association is primary; district is fallback. Membership and content queries
must apply profile, block, and community visibility policies.

### Notifications

Consumes versioned outbox events and owns notification preferences, in-app
notifications, delivery attempts, and provider tokens. Push/email providers sit
behind ports. Delivery is at-least-once, so handlers and provider requests need
deduplication keys.

### Media and moderation

Clients upload through short-lived presigned URLs, followed by a completion call
that validates ownership, MIME type, size, and checksum. Untrusted media enters a
pending/quarantined state. CPU-heavy thumbnail/transcode work runs in a worker,
not on the async request runtime. Moderation vendors must be replaceable adapters.

### Analytics

Product analytics consumes versioned domain events from the outbox. Event names
and properties are a governed contract. Operational logs are not product
analytics, and neither should contain raw secrets, private messages, injury text,
or unnecessary personal data.

### Admin

Use the same backend with a separately protected `/admin/v1` surface and explicit
RBAC/audit logging. Admin mutations call the owning domain's use cases rather than
writing tables directly. A separate admin deployment can come later if needed.

## PostgreSQL and migration rules

- Use SQLx migrations and check their checksums into source control.
- Applied migrations are immutable. Fix forward with a new migration.
- Prefer one meaningful schema change per migration.
- Use expand/migrate/contract changes for deployments that span app versions.
- Make constraints express real invariants: `NOT NULL`, foreign keys, unique and
  check constraints, and appropriate delete behavior.
- Add indexes from observed query shapes and verify with `EXPLAIN (ANALYZE, BUFFERS)`.
- Large indexes that require `CREATE INDEX CONCURRENTLY` belong in an explicitly
  non-transactional migration and need deployment awareness.
- Prefer text plus a check constraint or a reference table for states expected to
  evolve frequently. PostgreSQL enums are acceptable only for truly stable sets.
- Use `jsonb` for opaque/versioned payloads and event data, not as a substitute for
  relational columns that need constraints, joins, or indexes.
- All mutable tables have `created_at`; add `updated_at` only when its semantics are
  defined. Use explicit lifecycle timestamps such as `completed_at` where meaningful.
- Soft deletion is not universal. Use it only where recovery, audit, or product
  semantics require it; otherwise delete with explicit retention policy.
- Seed stable reference/content data through versioned, idempotent tooling. Keep
  sample development data separate from production reference data.

The initial extensions are `pg_trgm`, `cube`, and `earthdistance`; all are
available on the provisioned Railway database. Do not enable extensions simply
because they might be useful later.

## API conventions

- REST/JSON is the default public contract. WebSockets handle live, bidirectional
  events; they do not replace command/query HTTP endpoints.
- Version public routes under `/v1` and events with an explicit schema version.
- Use cursor/keyset pagination for growing collections.
- Accept an `Idempotency-Key` on retry-prone mutations.
- Use request IDs and propagate correlation/causation IDs through outbox jobs.
- Use explicit response DTOs; never serialize database row structs directly.
- Backward-compatible additions are preferred. Breaking contract changes require
  a new version or a deliberate migration plan.
- Health endpoints distinguish liveness from readiness. Readiness checks critical
  dependencies with tight timeouts.

## Testing expectations

- Domain rules receive fast, deterministic unit tests without a database.
- Repository tests run against real PostgreSQL with migrations applied.
- Each application use case tests authorization, invariant failures, transaction
  rollback, idempotency, and emitted events where relevant.
- API tests cover status, response shape, authentication, and error mapping rather
  than duplicating every domain rule.
- Time, ID generation, and external providers become injectable at the boundary
  when determinism is required; do not abstract them preemptively everywhere.
- Migration CI creates an empty database, applies all migrations, and runs SQLx
  offline-metadata checks.

## Definition of done

A backend change is complete when:

1. It lives in the owning domain and respects dependency direction.
2. Business invariants exist in domain code and database constraints where possible.
3. SQL is colocated under the owning Postgres adapter.
4. Authorization and privacy behavior are explicit.
5. Transactions, retries, duplicate delivery, and concurrency were considered.
6. Structured telemetry is useful without leaking sensitive data.
7. Appropriate unit/integration tests pass.
8. Migrations, API contracts, and this guide are updated when affected.

## Deliberately deferred decisions

Cloud provider, deployment platform, email/push vendors, analytics vendor, CDN,
and media moderation vendor should be selected against cost, target geography,
operational experience, and compliance needs. Keep these behind narrow adapters
until selected. Kubernetes, Kafka, Elasticsearch, and separate domain services are
not baseline requirements.
