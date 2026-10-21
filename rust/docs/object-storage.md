# Object storage

Spot uses Supabase Storage for media bytes and keeps durable application data,
ownership, and object metadata in the primary PostgreSQL database. Supabase is
not a second profile database and does not replace Better Auth.

## Configuration

The API requires:

- `SUPABASE_URL`
- `SUPABASE_SECRET_KEY`
- `SUPABASE_MAIN_BUCKET_NAME`

Use Supabase's modern server-only secret key. The Rust backend intentionally
does not use the publishable key or the legacy `anon` and `service_role` keys.
Secrets are loaded into `SecretString`, marked sensitive in HTTP headers, and
must never be sent to a mobile client or included in logs.

The configured bucket is currently public. Public access applies only to object
downloads; uploads and deletes must remain behind authenticated Spot API use
cases. Store only intentionally public assets such as profile avatars there.
Limitations, identity documents, private messages, and other sensitive user data
must not use this bucket.

## Ownership and paths

The integration lives under `shared/object_storage` because multiple domains may
eventually store media. Domain modules remain responsible for authorization,
file validation, size limits, object-key construction, and database updates.
The shared client provides narrowly scoped `upload`, `exists`, `replace`,
`delete`, and `public_url` operations. It does not decide who may perform them.

Avatar paths should be immutable and user-scoped, for example:

```text
avatars/{user_id}/{object_id}.webp
```

Replacing an avatar should create a new object key. Update the profile record in
PostgreSQL first through the profile use case, then remove the previous object
as retryable background cleanup. This avoids stale CDN content and losing the
current avatar when a multi-step operation partially fails.

The client's `replace` operation performs a same-key `PUT` for cases that truly
require it. Do not use it for normal avatar updates: Supabase advises creating a
new path because CDN propagation can temporarily serve stale content after an
overwrite.
