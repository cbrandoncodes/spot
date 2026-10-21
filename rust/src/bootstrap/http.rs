use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method, header},
};
use better_auth::integrations::axum::AxumIntegration;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::{AppState, modules};

pub fn router(
    state: AppState,
    trusted_origins: &[String],
) -> Result<Router, axum::http::header::InvalidHeaderValue> {
    let auth_routes = state.auth.clone().axum_router_with_state::<AppState>();
    let origins = trusted_origins
        .iter()
        .map(|origin| origin.parse::<HeaderValue>())
        .collect::<Result<Vec<_>, _>>()?;
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            header::ACCEPT,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("x-requested-with"),
        ]))
        .expose_headers([HeaderName::from_static("set-auth-token")])
        .allow_credentials(true);

    Ok(Router::new()
        .nest("/api/auth", auth_routes)
        .merge(modules::auth::http::routes())
        .merge(modules::health::http::routes())
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http()))
}
