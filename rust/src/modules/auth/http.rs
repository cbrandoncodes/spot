use axum::{Json, Router, routing::get};
use better_auth::integrations::axum::CurrentSession;
use better_auth::prelude::AuthUser;

use crate::AppState;

use super::schema::AppAuthSchema;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/me", get(me))
}

async fn me(session: CurrentSession<AppAuthSchema>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": session.user.id(),
        "email": session.user.email(),
        "name": session.user.name,
        "image": session.user.image,
    }))
}
