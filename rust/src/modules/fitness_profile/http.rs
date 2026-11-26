mod dto;
mod error;
mod options;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, patch, post},
};
use garde::Validate as _;

use crate::{AppState, modules::auth::extractor::AuthenticatedUser};

use self::{
    dto::{
        CompleteOnboardingRequest, FitnessProfileResponse, GoalsExperienceRequest,
        MovementWellnessRequest, OnboardingResponse, ReopenOnboardingRequest, ScheduleSetupRequest,
        TrainingPreferencesRequest,
    },
    error::ApiError,
    options::OnboardingOptionsResponse,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/onboarding", get(get_onboarding))
        .route("/api/onboarding/options", get(get_options))
        .route(
            "/api/onboarding/groups/goals-experience",
            patch(save_goals_experience),
        )
        .route(
            "/api/onboarding/groups/schedule-setup",
            patch(save_schedule_setup),
        )
        .route(
            "/api/onboarding/groups/training-preferences",
            patch(save_training_preferences),
        )
        .route(
            "/api/onboarding/groups/movement-wellness",
            patch(save_movement_wellness),
        )
        .route("/api/onboarding/complete", post(complete_onboarding))
        .route("/api/onboarding/reopen", post(reopen_onboarding))
        .route("/api/fitness-profile/me", get(get_fitness_profile))
}

async fn get_onboarding(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
) -> Result<Json<OnboardingResponse>, ApiError> {
    let draft = state
        .fitness_profiles
        .get_onboarding(identity.user_id)
        .await?;
    Ok(Json(OnboardingResponse::from(draft)))
}

async fn get_options(_identity: AuthenticatedUser) -> Json<OnboardingOptionsResponse> {
    Json(OnboardingOptionsResponse::new())
}

async fn save_goals_experience(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<GoalsExperienceRequest>,
) -> Result<Json<OnboardingResponse>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    let revision = request.revision;
    let answers = request.into_domain()?;
    let draft = state
        .fitness_profiles
        .save_goals_experience(identity.user_id, revision, answers)
        .await?;
    Ok(Json(OnboardingResponse::from(draft)))
}

async fn save_schedule_setup(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<ScheduleSetupRequest>,
) -> Result<Json<OnboardingResponse>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    let revision = request.revision;
    let answers = request.into_domain()?;
    let draft = state
        .fitness_profiles
        .save_schedule_setup(identity.user_id, revision, answers)
        .await?;
    Ok(Json(OnboardingResponse::from(draft)))
}

async fn save_training_preferences(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<TrainingPreferencesRequest>,
) -> Result<Json<OnboardingResponse>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    let revision = request.revision;
    let answers = request.into_domain()?;
    let draft = state
        .fitness_profiles
        .save_training_preferences(identity.user_id, revision, answers)
        .await?;
    Ok(Json(OnboardingResponse::from(draft)))
}

async fn save_movement_wellness(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<MovementWellnessRequest>,
) -> Result<Json<OnboardingResponse>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    let revision = request.revision;
    let answers = request.into_domain()?;
    let draft = state
        .fitness_profiles
        .save_movement_wellness(identity.user_id, revision, answers)
        .await?;
    Ok(Json(OnboardingResponse::from(draft)))
}

async fn complete_onboarding(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<CompleteOnboardingRequest>,
) -> Result<Json<FitnessProfileResponse>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    let profile = state
        .fitness_profiles
        .complete(identity.user_id, request.revision)
        .await?;
    Ok(Json(FitnessProfileResponse::from(profile)))
}

async fn reopen_onboarding(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<ReopenOnboardingRequest>,
) -> Result<Json<OnboardingResponse>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    let draft = state
        .fitness_profiles
        .reopen(identity.user_id, request.revision)
        .await?;
    Ok(Json(OnboardingResponse::from(draft)))
}

async fn get_fitness_profile(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
) -> Result<Json<FitnessProfileResponse>, ApiError> {
    let profile = state
        .fitness_profiles
        .get_profile(identity.user_id)
        .await?
        .ok_or_else(ApiError::profile_not_found)?;
    Ok(Json(FitnessProfileResponse::from(profile)))
}
