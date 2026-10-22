use axum::{Json, Router, routing::get};
use better_auth::prelude::AuthUser;

use crate::AppState;

use super::extractor::AuthenticatedUser;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/me", get(me))
}

async fn me(identity: AuthenticatedUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": identity.user.id(),
        "email": identity.user.email(),
        "name": identity.user.name,
        "image": identity.user.image,
    }))
}
