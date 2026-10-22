use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, patch, put},
};
use garde::Validate as _;
use serde::{Deserialize, Serialize};

use crate::{AppState, modules::auth::extractor::AuthenticatedUser};

use super::{
    domain::{
        Bio, City, CountryCode, District, FriendRequestPolicy, ProfileChanges, ProfileName,
        Username, ValidationError, Visibility,
    },
    infrastructure::postgres::{Model, PrivacyChanges},
    service::{ProfileError, ProfileService},
};

const MAX_AVATAR_BYTES: usize = 5 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    let own_profile_routes = Router::new()
        .route("/me", get(get_me).patch(update_me))
        .route("/me/privacy", patch(update_privacy))
        .route("/me/avatar", put(upload_avatar).delete(delete_avatar))
        .route(
            "/usernames/{username}/availability",
            get(username_availability),
        )
        .layer(DefaultBodyLimit::max(MAX_AVATAR_BYTES));

    Router::new()
        .nest("/api/profile", own_profile_routes)
        .route("/api/profiles/{username}", get(get_discoverable_profile))
}

async fn get_discoverable_profile(
    State(state): State<AppState>,
    _identity: AuthenticatedUser,
    Path(username): Path<String>,
) -> Result<Json<DiscoverableProfileResponse>, ApiError> {
    let profile = state
        .profiles
        .find_discoverable(&username)
        .await?
        .ok_or_else(ApiError::profile_not_found)?;
    Ok(Json(DiscoverableProfileResponse::new(
        &state.profiles,
        profile,
    )?))
}

async fn get_me(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    let (user_id, name) = current_identity(&identity);
    let profile = state.profiles.get_or_create(user_id, name).await?;
    Ok(Json(ProfileResponse::new(&state.profiles, profile)))
}

async fn update_me(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let (user_id, name) = current_identity(&identity);
    request.validate().map_err(ApiError::invalid_request)?;
    let changes = request.into_changes()?;
    let profile = state.profiles.update(user_id, name, changes).await?;
    Ok(Json(ProfileResponse::new(&state.profiles, profile)))
}

async fn update_privacy(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Json(request): Json<UpdatePrivacyRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let (user_id, name) = current_identity(&identity);
    request.validate().map_err(ApiError::invalid_request)?;
    let privacy = request.into_changes()?;
    let profile = state
        .profiles
        .update_privacy(user_id, name, privacy)
        .await?;
    Ok(Json(ProfileResponse::new(&state.profiles, profile)))
}

async fn username_availability(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    Path(username): Path<String>,
) -> Result<Json<UsernameAvailabilityResponse>, ApiError> {
    let (user_id, _) = current_identity(&identity);
    let available = state
        .profiles
        .username_available(user_id, &username)
        .await?;
    Ok(Json(UsernameAvailabilityResponse {
        username,
        available,
    }))
}

async fn upload_avatar(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ProfileResponse>, ApiError> {
    let (user_id, name) = current_identity(&identity);
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .ok_or_else(|| ApiError::invalid("contentType", "an image Content-Type is required"))?;
    let profile = state
        .profiles
        .upload_avatar(user_id, name, content_type, body.to_vec())
        .await?;
    Ok(Json(ProfileResponse::new(&state.profiles, profile)))
}

async fn delete_avatar(
    State(state): State<AppState>,
    identity: AuthenticatedUser,
) -> Result<Json<ProfileResponse>, ApiError> {
    let (user_id, name) = current_identity(&identity);
    let profile = state.profiles.delete_avatar(user_id, name).await?;
    Ok(Json(ProfileResponse::new(&state.profiles, profile)))
}

fn current_identity(identity: &AuthenticatedUser) -> (uuid::Uuid, &str) {
    let name = identity
        .user
        .name
        .typed()
        .ok()
        .and_then(Option::as_deref)
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Spot member");
    (identity.user_id, name)
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateProfileRequest {
    #[garde(length(chars, max = 256))]
    name: Option<String>,
    #[garde(length(bytes, max = 128))]
    username: Option<String>,
    #[garde(length(chars, max = 2048))]
    bio: Option<String>,
    #[garde(length(bytes, max = 16))]
    country_code: Option<String>,
    #[garde(length(chars, max = 256))]
    city: Option<String>,
    #[garde(length(chars, max = 256))]
    district: Option<String>,
}

impl UpdateProfileRequest {
    fn into_changes(self) -> Result<ProfileChanges, ApiError> {
        if self.name.is_none()
            && self.username.is_none()
            && self.bio.is_none()
            && self.country_code.is_none()
            && self.city.is_none()
            && self.district.is_none()
        {
            return Err(ApiError::invalid("body", "provide at least one field"));
        }

        Ok(ProfileChanges {
            name: self.name.map(ProfileName::try_from).transpose()?,
            username: self.username.map(Username::try_from).transpose()?,
            bio: self.bio.map(optional_domain_value::<Bio>).transpose()?,
            country_code: self
                .country_code
                .map(optional_domain_value::<CountryCode>)
                .transpose()?,
            city: self.city.map(optional_domain_value::<City>).transpose()?,
            district: self
                .district
                .map(optional_domain_value::<District>)
                .transpose()?,
        })
    }
}

fn optional_domain_value<T>(value: String) -> Result<Option<T>, ValidationError>
where
    T: TryFrom<String, Error = ValidationError>,
{
    if value.trim().is_empty() {
        Ok(None)
    } else {
        T::try_from(value).map(Some)
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdatePrivacyRequest {
    #[garde(length(bytes, max = 32))]
    visibility: Option<String>,
    discoverable_by_username: Option<bool>,
    show_primary_gym: Option<bool>,
    discoverable_in_gym: Option<bool>,
    show_district: Option<bool>,
    show_activity: Option<bool>,
    show_training_presence: Option<bool>,
    allow_gym_friend_requests: Option<bool>,
    show_body_metrics: Option<bool>,
    #[garde(length(bytes, max = 32))]
    friend_request_policy: Option<String>,
}

impl UpdatePrivacyRequest {
    fn into_changes(self) -> Result<PrivacyChanges, ApiError> {
        if self.visibility.is_none()
            && self.discoverable_by_username.is_none()
            && self.show_primary_gym.is_none()
            && self.discoverable_in_gym.is_none()
            && self.show_district.is_none()
            && self.show_activity.is_none()
            && self.show_training_presence.is_none()
            && self.allow_gym_friend_requests.is_none()
            && self.show_body_metrics.is_none()
            && self.friend_request_policy.is_none()
        {
            return Err(ApiError::invalid("body", "provide at least one field"));
        }

        Ok(PrivacyChanges {
            visibility: self
                .visibility
                .as_deref()
                .map(str::parse::<Visibility>)
                .transpose()?
                .map(|value| value.as_str().to_owned()),
            discoverable_by_username: self.discoverable_by_username,
            show_primary_gym: self.show_primary_gym,
            discoverable_in_gym: self.discoverable_in_gym,
            show_district: self.show_district,
            show_activity: self.show_activity,
            show_training_presence: self.show_training_presence,
            allow_gym_friend_requests: self.allow_gym_friend_requests,
            show_body_metrics: self.show_body_metrics,
            friend_request_policy: self
                .friend_request_policy
                .as_deref()
                .map(str::parse::<FriendRequestPolicy>)
                .transpose()?
                .map(|value| value.as_str().to_owned()),
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileResponse {
    user_id: String,
    name: String,
    username: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
    country_code: Option<String>,
    city: Option<String>,
    district: Option<String>,
    privacy: PrivacyResponse,
    created_at: String,
    updated_at: String,
}

impl ProfileResponse {
    fn new(service: &ProfileService, profile: Model) -> Self {
        let avatar_url = service.avatar_url(&profile);
        Self {
            user_id: profile.user_id.to_string(),
            name: profile.name,
            username: profile.username,
            avatar_url,
            bio: profile.bio,
            country_code: profile.country_code,
            city: profile.city,
            district: profile.district,
            privacy: PrivacyResponse {
                visibility: profile.visibility,
                discoverable_by_username: profile.discoverable_by_username,
                show_primary_gym: profile.show_primary_gym,
                discoverable_in_gym: profile.discoverable_in_gym,
                show_district: profile.show_district,
                show_activity: profile.show_activity,
                show_training_presence: profile.show_training_presence,
                allow_gym_friend_requests: profile.allow_gym_friend_requests,
                show_body_metrics: profile.show_body_metrics,
                friend_request_policy: profile.friend_request_policy,
            },
            created_at: profile.created_at.to_rfc3339(),
            updated_at: profile.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrivacyResponse {
    visibility: String,
    discoverable_by_username: bool,
    show_primary_gym: bool,
    discoverable_in_gym: bool,
    show_district: bool,
    show_activity: bool,
    show_training_presence: bool,
    allow_gym_friend_requests: bool,
    show_body_metrics: bool,
    friend_request_policy: String,
}

#[derive(Serialize)]
struct UsernameAvailabilityResponse {
    username: String,
    available: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoverableProfileResponse {
    user_id: String,
    name: String,
    username: String,
    avatar_url: Option<String>,
    bio: Option<String>,
    visibility: String,
    country_code: Option<String>,
    city: Option<String>,
    district: Option<String>,
}

impl DiscoverableProfileResponse {
    fn new(service: &ProfileService, profile: Model) -> Result<Self, ApiError> {
        let show_location = profile.show_district;
        let username = profile
            .username
            .clone()
            .ok_or_else(ApiError::invalid_profile_state)?;
        let avatar_url = service.avatar_url(&profile);
        Ok(Self {
            user_id: profile.user_id.to_string(),
            name: profile.name,
            username,
            avatar_url,
            bio: profile.bio,
            visibility: profile.visibility,
            country_code: show_location.then_some(profile.country_code).flatten(),
            city: show_location.then_some(profile.city).flatten(),
            district: show_location.then_some(profile.district).flatten(),
        })
    }
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    detail: String,
    field: Option<&'static str>,
}

impl ApiError {
    fn invalid(field: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "invalid_profile",
            detail: detail.into(),
            field: Some(field),
        }
    }

    fn invalid_request(error: garde::Report) -> Self {
        Self::invalid("body", error.to_string())
    }

    fn profile_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "profile_not_found",
            detail: "Profile was not found.".to_owned(),
            field: None,
        }
    }

    fn invalid_profile_state() -> Self {
        tracing::error!("discoverable profile is missing its username");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "profile_unavailable",
            detail: "Profile is temporarily unavailable.".to_owned(),
            field: None,
        }
    }
}

impl From<super::domain::ValidationError> for ApiError {
    fn from(error: super::domain::ValidationError) -> Self {
        Self::invalid(error.field, error.message)
    }
}

impl From<ProfileError> for ApiError {
    fn from(error: ProfileError) -> Self {
        match error {
            ProfileError::Validation(error) => error.into(),
            ProfileError::UsernameTaken => Self {
                status: StatusCode::CONFLICT,
                code: "username_taken",
                detail: "Username is already taken. Try another.".to_owned(),
                field: Some("username"),
            },
            ProfileError::InvalidAvatar(detail) => Self::invalid("avatar", detail),
            ProfileError::Repository(error) => {
                tracing::error!(error = %error, "profile repository operation failed");
                Self {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "profile_unavailable",
                    detail: "Profile is temporarily unavailable.".to_owned(),
                    field: None,
                }
            }
            ProfileError::Storage(error) => {
                tracing::error!(error = %error, "profile avatar storage operation failed");
                Self {
                    status: StatusCode::BAD_GATEWAY,
                    code: "avatar_storage_unavailable",
                    detail: "Avatar storage is temporarily unavailable.".to_owned(),
                    field: None,
                }
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ProblemDetails {
            problem_type: format!("https://spot.app/problems/{}", self.code),
            title: self.status.canonical_reason().unwrap_or("Request failed"),
            status: self.status.as_u16(),
            detail: self.detail,
            code: self.code,
            field: self.field,
        };
        (
            self.status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDetails {
    #[serde(rename = "type")]
    problem_type: String,
    title: &'static str,
    status: u16,
    detail: String,
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<&'static str>,
}
