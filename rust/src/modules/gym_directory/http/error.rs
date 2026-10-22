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
    pub(super) fn invalid_request(error: garde::Report) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "invalid_gym_search",
            detail: error.to_string(),
            field: Some("body"),
        }
    }
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::Validation(error) => error.into(),
            ServiceError::BranchNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "gym_branch_not_found",
                detail: "Gym branch was not found.".to_owned(),
                field: None,
            },
            ServiceError::OrganizationNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "gym_not_found",
                detail: "Gym was not found.".to_owned(),
                field: None,
            },
            ServiceError::GroupNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "gym_group_not_found",
                detail: "Official gym group was not found.".to_owned(),
                field: None,
            },
            ServiceError::Repository(error) => {
                tracing::error!(error = %error, "gym directory repository operation failed");
                Self {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "gym_directory_unavailable",
                    detail: "Gym directory is temporarily unavailable.".to_owned(),
                    field: None,
                }
            }
        }
    }
}

impl From<ValidationError> for ApiError {
    fn from(error: ValidationError) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "invalid_gym_search",
            detail: error.message,
            field: Some(error.field),
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
