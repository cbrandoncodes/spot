pub mod bootstrap;
pub mod migration;
pub mod modules;
pub mod shared;

use std::sync::Arc;

use axum::extract::FromRef;
use better_auth::{BetterAuth, seaorm::DatabaseConnection};

use crate::modules::auth::schema::AppAuthSchema;
use crate::modules::fitness_profile::FitnessProfileService;
use crate::modules::profile::ProfileService;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: Arc<BetterAuth<AppAuthSchema>>,
    pub fitness_profiles: FitnessProfileService,
    pub profiles: ProfileService,
}

impl FromRef<AppState> for Arc<BetterAuth<AppAuthSchema>> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}
