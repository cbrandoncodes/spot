# Onboarding Completion Concurrency Diagnosis

## Current application behavior

Spot completes onboarding by validating the client's expected draft revision,
locking the matching PostgreSQL row, and then writing the normalized fitness
profile and an immutable snapshot in one transaction. When two identical
completion requests overlap, one request can succeed while the other returns a
revision conflict after waiting for the same row. The purpose of this report is
to capture live evidence of that timing-dependent behavior, confirm the durable
database result, and define the requirements for the follow-up fix.

## 1. Which operation was tested?

Answer: `[HTTP method] [endpoint path]`


## 2. Which draft and command were used?

Answer: `user_id=[UUID]; submitted_revision=[integer]; identical_requests=[yes/no]`


## 3. What did each concurrent request return?

Answer: `request_a=[HTTP status]; request_b=[HTTP status]`


## 4. Was a database lock wait observed?

Answer: `[yes/no]; waiting_pid=[integer or none]; blocking_pid=[integer or none]`


## 5. What was the PostgreSQL transaction state?

Answer: `isolation=[value]; wait_event_type=[value]; wait_event=[value]`


## 6. What durable state remained after both requests?

Answer: `profile_revision=[integer]; snapshots_for_revision=[integer]`


## 7. Did the draft advance more than once?

Answer: `[yes/no]; revision_before=[integer]; revision_after=[integer]`


## 8. What timing-dependent defect was reproduced?

Answer in one sentence: `[observed behavior]`


## 9. Which code or query causes the defect?

Answer in one sentence: `[file/function/query and reason]`


## 10. What must Task 02-02 prove?

Answer: `concurrent_statuses=[...]; same_profile_revision=[yes/no]; snapshot_count=[integer]; stale_revision_status=[HTTP status]`
