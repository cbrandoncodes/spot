use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::super::{domain::ValidationError, service::ServiceError};

pub(super) struct ApiError {
    status: StatusCode,
    code: &'static str,
    detail: String,
    field: Option<&'static str>,
}

impl ApiError {
    fn invalid(field: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "invalid_onboarding",
            detail: detail.into(),
            field: Some(field),
        }
    }

    pub(super) fn invalid_request(error: garde::Report) -> Self {
        Self::invalid("body", error.to_string())
    }

    pub(super) fn profile_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "fitness_profile_not_found",
            detail: "Complete onboarding before requesting a fitness profile.".to_owned(),
            field: None,
        }
    }
}

impl From<ValidationError> for ApiError {
    fn from(error: ValidationError) -> Self {
        Self::invalid(error.field, error.message)
    }
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Validation(error) => error.into(),
            ServiceError::RevisionConflict => Self {
                status: StatusCode::CONFLICT,
                code: "onboarding_revision_conflict",
                detail: "Onboarding answers changed. Reload before saving again.".to_owned(),
                field: Some("revision"),
            },
            ServiceError::QuestionnaireVersionUnsupported => Self {
                status: StatusCode::CONFLICT,
                code: "onboarding_version_unsupported",
                detail: "This onboarding draft must be refreshed before it can be completed."
                    .to_owned(),
                field: None,
            },
            ServiceError::InvalidStoredDraft(error) => {
                tracing::error!(error = %error, "stored onboarding draft is invalid");
                internal_error()
            }
            ServiceError::CompletedProfileMissing => {
                tracing::error!("completed onboarding has no fitness profile");
                internal_error()
            }
            ServiceError::OnboardingNotComplete => Self {
                status: StatusCode::CONFLICT,
                code: "onboarding_not_complete",
                detail: "Only completed onboarding can be reopened.".to_owned(),
                field: None,
            },
            ServiceError::Repository(error) => {
                tracing::error!(error = %error, "fitness profile repository operation failed");
                internal_error()
            }
        }
    }
}

fn internal_error() -> ApiError {
    ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        code: "fitness_profile_unavailable",
        detail: "Fitness profile is temporarily unavailable.".to_owned(),
        field: None,
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
