use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;

use crate::{AppState, modules::health::infrastructure::postgres};

pub fn routes() -> Router<AppState> {
    let routes = Router::new()
        .route("/live", get(liveness))
        .route("/ready", get(readiness));

    Router::new().nest("/health", routes)
}

async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn readiness(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    postgres::is_ready(&state.db)
        .await
        .map(|()| Json(HealthResponse { status: "ok" }))
        .map_err(|error| {
            tracing::error!(error = %error, "database readiness check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                }),
            )
        })
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}
