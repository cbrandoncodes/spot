# Onboarding Completion Concurrency Diagnosis

## Current application behavior

Spot completes onboarding by validating the client's expected draft revision,
locking the matching PostgreSQL row, and then writing the normalized fitness
profile and an immutable snapshot in one transaction. When two identical
completion requests overlap, one request can succeed while the other returns a
revision conflict after waiting for the same row. The purpose of this report is
to capture live evidence of that timing-dependent behavior, confirm the durable
database result, and define the requirements for the follow-up fix.

## 1. Which command was tested?

Answer: `operation=[HTTP method and path]; user_id=[UUID]; submitted_revision=[integer]; identical_requests=[yes/no]`

## 2. What did the concurrent requests return?

Answer: `request_a=[HTTP status]; request_b=[HTTP status]`

## 3. What lock evidence did PostgreSQL show?

Answer: `isolation=[value]; waiting_pid=[integer or none]; blocking_pid=[integer or none]; wait_event=[value]`

## 4. What durable state remained after both requests?

Answer: `revision_before=[integer]; revision_after=[integer]; duplicate_write=[yes/no]`

## 5. What was diagnosed, and what must the fix prove?

Answer: `cause=[value]; expected_concurrent_statuses=[...]; same_profile_revision=[yes/no]; stale_revision_status=[HTTP status]`
