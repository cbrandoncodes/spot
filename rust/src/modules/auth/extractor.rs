use std::sync::Arc;

use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use better_auth::{
    BetterAuth,
    integrations::axum::CurrentSession,
    prelude::{SessionView, UserView},
};
use serde::Serialize;
use uuid::Uuid;

use super::schema::AppAuthSchema;

/// Spot's authenticated request identity.
///
/// This delegates session resolution to Better Auth exactly once, then converts
/// its string user ID into the UUID used by Spot's domain and persistence code.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub user: UserView,
    pub session: SessionView,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<BetterAuth<AppAuthSchema>>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let current = CurrentSession::<AppAuthSchema>::from_request_parts(parts, state).await?;
        let user_id = current
            .user
            .id
            .as_str()
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or_else(invalid_identity)?;

        Ok(Self {
            user_id,
            user: current.user,
            session: current.session,
        })
    }
}

fn invalid_identity() -> Response {
    let status = StatusCode::UNAUTHORIZED;
    let body = ProblemDetails {
        problem_type: "https://spot.app/problems/invalid_session",
        title: "Unauthorized",
        status: status.as_u16(),
        detail: "The authenticated session does not contain a valid user identity.",
        code: "invalid_session",
    };

    (
        status,
        [(header::CONTENT_TYPE, "application/problem+json")],
        Json(body),
    )
        .into_response()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDetails {
    #[serde(rename = "type")]
    problem_type: &'static str,
    title: &'static str,
    status: u16,
    detail: &'static str,
    code: &'static str,
}
