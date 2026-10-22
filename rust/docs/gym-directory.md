# Gym Directory and Official Groups

The gym directory separates a gym organization from its physical branches. A
brand owns one or more branches; coordinates, addresses, opening hours,
facilities, equipment and membership plans belong to a branch.

Official gym groups are a separate bounded context from user-created social
groups. The first slice exposes official group profiles and published posts but
does not yet allow membership or operator mutations.

## Location privacy

Spot requests device coordinates only when the user explicitly invokes nearby
discovery. Coordinates are sent in a JSON request body, used for that query and
discarded. They must never be persisted to a user, profile, session, analytics
event or application log. Spot does not perform background or continuous
location tracking.

The existing PostgreSQL `cube` and `earthdistance` extensions provide indexed
spherical distance queries. This works on the current Railway PostgreSQL image
without moving the database to a PostGIS-specific deployment. If directory
scale or polygon/geofence requirements justify PostGIS later, the repository is
the only application layer that should change.

## Discovery API

All current endpoints require `AuthenticatedUser` and therefore accept either a
Better Auth session cookie or bearer token.

### `POST /api/gym-discovery/search`

The same endpoint supports manual and nearby discovery. Omitting `location`
performs a manual directory search.

```json
{
  "query": "Fitness Central",
  "area": "Lekki",
  "location": {
    "latitude": 6.4474,
    "longitude": 3.4723,
    "radiusMeters": 10000
  },
  "facilities": ["parking", "showers"],
  "equipment": ["squat_rack", "cable_station"],
  "limit": 20,
  "offset": 0
}
```

All supplied facility and equipment filters must match. Results are ordered by
distance when location is supplied, then verified status and name. Distance is
returned in metres. The maximum radius is 50 km and the maximum page size is
50.

The location is intentionally carried in a POST body rather than query
parameters so precise coordinates do not enter browser history, ordinary proxy
access logs or URL analytics.

### `GET /api/gym-discovery/options`

Returns the controlled facility and equipment codes accepted by search.

### `GET /api/gyms/{slug}`

Returns a public organization profile and its public branches.

### `GET /api/gym-branches/{branchId}`

Returns a branch profile containing its organization, address, coordinates,
media, standard hours, upcoming hour exceptions, facilities, equipment
inventory, active membership plans, links and official-group summary.

### `GET /api/gym-groups/{groupId}`

Returns public or unlisted official-group metadata and its branch context.
Private groups are not exposed by this endpoint.

### `GET /api/gym-groups/{groupId}/posts?limit=20&offset=0`

Returns published official posts ordered by pinned status and publication time.
Supported post types are `announcement`, `maintenance`, `class_event`,
`promotion`, `hours_change`, `new_equipment` and `membership`.

## Data rules

- Organizations and branches have independent visibility and lifecycle state.
- Only public branches appear in discovery. Unlisted branches can be accessed
  directly; private branches cannot.
- Organization verification is explicit and never inferred from user input.
- Money uses integer minor units plus an ISO-style uppercase currency code.
- Holiday and temporary hour changes are structured exceptions; announcements
  alone do not alter opening-hour truth.
- Equipment uses the same stable codes accepted by fitness onboarding.
- User-entered filter codes are validated before reaching SQL, and all query
  values are bound parameters.

## Next slice

The next slice should add verified organization operators, ownership claims,
group memberships, hashed invitations, optional physical-membership
verification and privacy-safe member discovery. Joining a group must never
create friendships or expose a member in the directory automatically.
