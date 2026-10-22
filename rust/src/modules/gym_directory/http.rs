mod dto;
mod error;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use garde::Validate as _;
use uuid::Uuid;

use crate::{AppState, modules::auth::extractor::AuthenticatedUser};

use self::{
    dto::{CatalogOptionsResponse, GroupPostsQuery, SearchGymsRequest},
    error::ApiError,
};
use super::domain::{
    GroupPostPage, GymBranchProfile, GymOrganizationProfile, GymSearchPage, OfficialGroupProfile,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/gym-discovery/search", post(search))
        .route("/api/gym-discovery/options", get(options))
        .route("/api/gyms/{slug}", get(organization))
        .route("/api/gym-branches/{branch_id}", get(branch))
        .route("/api/gym-groups/{group_id}", get(group))
        .route("/api/gym-groups/{group_id}/posts", get(group_posts))
}

async fn organization(
    State(state): State<AppState>,
    _identity: AuthenticatedUser,
    Path(slug): Path<String>,
) -> Result<Json<GymOrganizationProfile>, ApiError> {
    Ok(Json(state.gyms.organization(&slug).await?))
}

async fn search(
    State(state): State<AppState>,
    _identity: AuthenticatedUser,
    Json(request): Json<SearchGymsRequest>,
) -> Result<Json<GymSearchPage>, ApiError> {
    request.validate().map_err(ApiError::invalid_request)?;
    Ok(Json(state.gyms.search(request.into_domain()).await?))
}

async fn options(_identity: AuthenticatedUser) -> Json<CatalogOptionsResponse> {
    Json(CatalogOptionsResponse::new())
}

async fn branch(
    State(state): State<AppState>,
    _identity: AuthenticatedUser,
    Path(branch_id): Path<Uuid>,
) -> Result<Json<GymBranchProfile>, ApiError> {
    Ok(Json(state.gyms.branch(branch_id).await?))
}

async fn group(
    State(state): State<AppState>,
    _identity: AuthenticatedUser,
    Path(group_id): Path<Uuid>,
) -> Result<Json<OfficialGroupProfile>, ApiError> {
    Ok(Json(state.gyms.group(group_id).await?))
}

async fn group_posts(
    State(state): State<AppState>,
    _identity: AuthenticatedUser,
    Path(group_id): Path<Uuid>,
    Query(query): Query<GroupPostsQuery>,
) -> Result<Json<GroupPostPage>, ApiError> {
    query.validate().map_err(ApiError::invalid_request)?;
    Ok(Json(
        state
            .gyms
            .group_posts(group_id, query.limit, query.offset)
            .await?,
    ))
}
