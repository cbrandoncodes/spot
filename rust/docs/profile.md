# Profile

Profile owns a user's public identity and social privacy settings. Authentication
continues to own credentials and sessions; Fitness Profile will own any
demographic or limitation data needed for recommendations; Gym will own gym
records and membership links.

The first authenticated request lazily creates a private profile from the auth
user's name. Username selection remains explicit.

## Endpoints

All endpoints require a Better Auth session cookie or bearer token.

### `GET /api/profile/me`

Returns the current user's complete profile and privacy settings.

### `PATCH /api/profile/me`

Accepts any non-empty subset of:

```json
{
  "name": "Ada Okafor",
  "username": "ada.trains",
  "bio": "Learning to lift well.",
  "countryCode": "NG",
  "city": "Lagos",
  "district": "Yaba"
}
```

Sending an empty string clears `bio`, `countryCode`, `city`, or `district`.
Usernames are 3–30 ASCII letters, numbers, periods, or underscores; they start
and end with a letter or number, cannot repeat separators, and are unique without
regard to ASCII letter case.

### `GET /api/profile/usernames/{username}/availability`

Validates a candidate and reports whether another profile owns it. The current
user's existing username remains available to that user. The unique database
index is authoritative if concurrent requests race.

### `GET /api/profiles/{username}`

Returns the basic profile of an authenticated, discoverable user. Profiles with
username discovery disabled return `404`, avoiding a separate endpoint that
confirms their existence. Name, username, avatar, and bio form the basic profile;
country, city, and district are returned only when `showDistrict` is enabled.
Workout activity, live training presence, body metrics, and gym membership are
not returned by this endpoint.

### `PATCH /api/profile/me/privacy`

Accepts any non-empty subset of:

```json
{
  "visibility": "private",
  "discoverableByUsername": true,
  "showPrimaryGym": false,
  "discoverableInGym": false,
  "showDistrict": false,
  "showActivity": false,
  "showTrainingPresence": false,
  "allowGymFriendRequests": false,
  "showBodyMetrics": false,
  "friendRequestPolicy": "anyone"
}
```

Visibility is `public` or `private`. Friend-request policy is `anyone`,
`friends_of_friends`, `same_gym`, or `no_one`. Location, gym, activity, body
metrics, and live training presence default hidden.

### `PUT /api/profile/me/avatar`

The request body is the raw image, not JSON or multipart. Set `Content-Type` to
`image/jpeg`, `image/png`, or `image/webp`. The maximum request size is 5 MB and
the image signature must match the declared type.

Each upload uses a new `avatars/{user_id}/{uuid}.ext` object key. The profile row
stores only the key and responses derive the public URL from storage settings.

### `DELETE /api/profile/me/avatar`

Clears the profile's avatar reference and removes the previous object. Storage
cleanup failure does not restore a deleted profile reference; it is logged for
operational cleanup.

## Deferred fields

- `primaryGymId` arrives with the Gym module so a real foreign key can enforce it.
- Age range and gender belong to Fitness Profile only if recommendation logic
  needs them. They are not general social-profile fields.
- Friendship-protected activity and gym data wait for those domain modules. The
  basic discoverable profile endpoint intentionally does not expose them.
