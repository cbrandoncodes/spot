# Onboarding and Fitness Profile

Onboarding is a resumable workflow that produces Spot's private fitness profile.
It is not part of the public/social profile and none of its answers are returned
by profile discovery endpoints.

The questionnaire currently has version `1` and four stable group keys:

1. `goals_experience`
2. `schedule_setup`
3. `training_preferences`
4. `movement_wellness`

Group boundaries are an API and product-flow concern. Partial answers are stored
as a typed JSONB draft; completing onboarding atomically writes normalized
relational tables and an immutable recommendation-input snapshot.

## Concurrency and completion

Every draft response contains a `revision`. Send that revision with the next
group update. A stale revision returns `409 onboarding_revision_conflict` rather
than overwriting newer answers from another device.

Completion requires all four groups and runs in one database transaction. It:

1. locks and verifies the draft revision;
2. creates or replaces the normalized current fitness profile;
3. increments the fitness-profile revision;
4. writes ranked goals, equipment, preferences, avoidances and constraints;
5. creates an immutable `fitness_profile_snapshots` row; and
6. marks the draft complete.

Retrying the same completion request after a lost response returns the already
completed profile. Editing any group clears the draft's completion marker; a
subsequent completion creates a new profile revision and snapshot.

## Endpoints

All endpoints require a Better Auth session cookie or bearer token.

### `GET /api/onboarding/options`

Returns supported codes and display labels for goals, experience, activity,
environment, equipment, workout types, exercise avoidances, coaching style,
movement constraints, weekdays and unit systems. Clients should submit codes,
not labels.

The initial exercise list is deliberately small. These stable codes bridge the
MVP until Exercise Content owns the complete exercise/equipment catalog. Stored
codes remain parseable even if an option is no longer offered to new users.

### `GET /api/onboarding`

Creates an empty draft on first access and returns its answers and group status.

### `PATCH /api/onboarding/groups/goals-experience`

```json
{
  "revision": 0,
  "primaryGoal": "build_muscle",
  "secondaryGoals": ["get_stronger"],
  "experienceLevel": "beginner",
  "currentActivityLevel": "lightly_active"
}
```

The primary goal is required. Up to two unique secondary goals are allowed.

### `PATCH /api/onboarding/groups/schedule-setup`

```json
{
  "revision": 1,
  "daysPerWeek": 3,
  "minimumDurationMinutes": 30,
  "preferredDurationMinutes": 45,
  "maximumDurationMinutes": 60,
  "trainingEnvironment": "gym",
  "availableEquipment": ["dumbbells", "adjustable_bench", "cable_station"],
  "preferredWeekdays": ["monday", "wednesday", "friday"]
}
```

Days are restricted to `1..=7`; every duration is `15..=180` minutes and must
satisfy `minimum <= preferred <= maximum`. Equipment must contain at least one
unique supported code. Preferred weekdays are optional; when supplied, their
count must be at least the requested training frequency.

### `PATCH /api/onboarding/groups/training-preferences`

```json
{
  "revision": 2,
  "preferredWorkoutTypes": ["hypertrophy", "strength"],
  "dislikedExercises": [
    {
      "exerciseCode": "burpee",
      "preference": "never_include"
    }
  ],
  "coachingStyle": "balanced",
  "startingIntensity": "gentle",
  "consistencyBarrier": "My work schedule changes each week."
}
```

`prefer_alternative` is a soft preference. `never_include` is a hard user
exclusion, but remains distinct from a movement limitation.

### `PATCH /api/onboarding/groups/movement-wellness`

```json
{
  "revision": 3,
  "movementConstraints": [
    {
      "bodyRegion": "knee",
      "movementPattern": "squat",
      "impact": "modify",
      "notes": "Deep knee flexion is uncomfortable."
    }
  ],
  "accessibilityNeeds": null,
  "heightMm": null,
  "weightGrams": null,
  "unitSystem": "metric",
  "acknowledgeSensitiveData": true
}
```

Constraints describe how movement is affected; Spot does not infer or store a
medical diagnosis. `movementPattern` is optional. `monitor`, `modify` and
`avoid` have progressively stronger recommendation precedence.

Acknowledgement is required when the group contains a constraint,
accessibility information, height or weight. Height and weight remain optional
and are stored as integer millimetres and grams rather than floating-point
values.

### `POST /api/onboarding/complete`

```json
{
  "revision": 4
}
```

Returns the completed private fitness profile and its revision.

### `POST /api/onboarding/reopen`

```json
{
  "revision": 5
}
```

Reopens a completed onboarding draft without deleting its answers, current
fitness profile or immutable snapshots. The request must contain the latest
draft revision. It clears `completedAt`, advances the draft revision and returns
the draft so it can be completed again. A draft that is not complete returns
`409 onboarding_not_complete`.

### `GET /api/fitness-profile/me`

Returns the authenticated user's normalized current fitness profile. It returns
`404` until onboarding has been completed. Editing uses the same typed group
endpoints followed by another completion, ensuring edits produce a new immutable
snapshot.

## Privacy and logging

- Fitness-profile data never enters public profile responses.
- Movement constraints, accessibility information and measurements must not be
  written to application logs, analytics events or social activity.
- Only the authenticated owner can use these endpoints.
- The current sensitive-data notice identifier is stored alongside the consent
  timestamp when sensitive values are provided.
- Account deletion cascades from `auth_users` through drafts, the current
  profile and all snapshots.
- Free-form fields are intentionally short and should describe training impact,
  not detailed medical history.

## Recommendation contract

Program selection must consume an immutable fitness-profile snapshot and record
both its ID and the recommendation ruleset version. Recommendation precedence is:

```text
movement safety constraints
  > hard exercise exclusions
  > equipment availability
  > schedule feasibility
  > ranked goals
  > soft preferences
```

The onboarding module does not select or generate programs. It produces a
validated, explainable input for the future deterministic recommendation engine.
