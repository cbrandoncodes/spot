use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{AppState, modules};

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(modules::health::http::routes())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
