pub mod bootstrap;
pub mod migration;
pub mod modules;

use std::sync::Arc;

use axum::extract::FromRef;
use better_auth::{BetterAuth, seaorm::DatabaseConnection};

use crate::modules::auth::schema::AppAuthSchema;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: Arc<BetterAuth<AppAuthSchema>>,
}

impl FromRef<AppState> for Arc<BetterAuth<AppAuthSchema>> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}
