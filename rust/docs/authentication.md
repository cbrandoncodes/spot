# Authentication

Spot uses `better-auth-rs` pinned to Git revision
`23c89f4e51999fe5767804879c82c60249cf9f84`. The project owns the SeaORM
entities and migrations; upgrading Better Auth requires regenerating its schema,
reviewing the diff, and adding a new migration.

## Supported MVP flows

- Email and password sign-up/sign-in with mandatory email verification.
- Six-digit email OTP sign-in, verification, password reset, and email changes,
  delivered through Resend and stored only as hashes.
- Opaque database sessions through secure cookies or bearer tokens.
- Session listing and revocation.
- Password change for authenticated users.
- Google OAuth when both Google environment variables are configured.
- Safe account linking: different-email linking and unlinking the final account
  are forbidden, and implicit linking to an unverified local email is rejected.
- Authenticated account deletion.
- Database-backed per-endpoint rate limiting.

Apple sign-in is intentionally not configured yet. Public usernames belong to
the profile module, not authentication.

## Routes

Better Auth is mounted at `/api/auth`. Common routes are:

- `POST /api/auth/sign-up/email`
- `POST /api/auth/sign-in/email`
- `POST /api/auth/email-otp/send-verification-otp`
- `POST /api/auth/sign-in/email-otp`
- `POST /api/auth/email-otp/verify-email`
- `POST /api/auth/email-otp/request-password-reset`
- `POST /api/auth/email-otp/reset-password`
- `POST /api/auth/email-otp/request-email-change`
- `POST /api/auth/email-otp/change-email`
- `GET /api/auth/get-session`
- `POST /api/auth/sign-out`
- `GET /api/auth/list-sessions`
- `POST /api/auth/revoke-session`
- `POST /api/auth/revoke-other-sessions`
- `POST /api/auth/change-password`
- `GET /api/auth/list-accounts`
- `POST /api/auth/unlink-account`
- `POST /api/auth/delete-user`
- `POST /api/auth/sign-in/social` when Google is configured

`GET /api/me` is the Spot-owned protected-route example. It accepts the session
cookie or `Authorization: Bearer <session-token>`.

## Protected Spot routes

Spot-owned private handlers use the `AuthenticatedUser` Axum extractor. It
delegates cookie or bearer-token resolution to Better Auth's `CurrentSession`
exactly once, validates the application's UUID user ID, and makes the resolved
Better Auth user and session available to the handler. Do not combine it with a
second authentication middleware or `CurrentSession`, since that could repeat
session resolution.

Public routes must be intentionally separated and documented. New private
routes must accept `AuthenticatedUser`; never parse session cookies, bearer
tokens, or auth database rows inside domain services or repositories.

## Running locally

1. Install PostgreSQL and ICU 78. On Apple Silicon with Homebrew, install
   `icu4c@78`; `.cargo/config.toml` provides its linker path.
2. Copy `.env.example` to `.env`, replace the database URL and auth secret, and
   configure Resend. Optionally add both Google values.
3. Run migrations with `cargo run --bin migrate`.
4. Start the API with `cargo run --bin api`.

For Google, register `{SPOT_AUTH_BASE_URL}/api/auth/callback/google` as an
authorized redirect URI.

## Authentication email templates

Authentication emails have separate HTML and plain-text templates in
`templates/email/auth_otp.html` and `templates/email/auth_otp.txt`. They are
compiled into the API binary and rendered by the auth infrastructure, while the
Resend adapter is responsible only for delivery. Keep product copy and visual
changes in those template files rather than embedding markup in provider code.
Their colors, typography, shape, hierarchy, and voice must follow the repository
root `Spot-DESIGN.md`, which is the authoritative design reference.

The renderer supplies purpose-specific copy for sign-in, email verification,
password reset, and email changes. Any value inserted into HTML must remain
escaped, and every email must retain an equivalent plain-text version.

The approved logo is retained in the repository root `assets/` directory and
delivered to email clients from ImageKit. Keep the CDN URL centralized in the
email renderer rather than repeating it in templates or provider adapters.

Local/test environments run migrations on API startup by default. Staging and
production should run the migration binary as a release step and leave
`SPOT_DATABASE_RUN_MIGRATIONS=false`.

Production images must provide ICU 78 and its `icuuc`, `icui18n`, and `icudata`
shared libraries. This is a current Better Auth dependency, not a Spot domain
requirement.
