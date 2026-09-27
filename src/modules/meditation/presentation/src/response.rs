//! Error mapping and the same-origin guard.

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::ApplicationError;

/// Typed handler error.
pub enum ApiError {
    StatusCode(StatusCode),
    Application(ApplicationError),
}

impl ApiError {
    pub fn unauthorized() -> Self {
        Self::StatusCode(StatusCode::UNAUTHORIZED)
    }

    pub fn forbidden() -> Self {
        Self::StatusCode(StatusCode::FORBIDDEN)
    }
}

/// Maps application errors onto the API contract (404 stays generic, no
/// internal detail leaks).
pub fn application_error_response(error: ApplicationError) -> Response {
    let status = match &error {
        ApplicationError::UnsupportedMedia => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApplicationError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ApplicationError::NotFound => StatusCode::NOT_FOUND,
        ApplicationError::DuplicateCategory => StatusCode::CONFLICT,
        ApplicationError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        ApplicationError::Storage(_) => StatusCode::SERVICE_UNAVAILABLE,
        ApplicationError::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = match &error {
        ApplicationError::Repository(_) | ApplicationError::Storage(_) => {
            "internal error".to_owned()
        }
        other => other.to_string(),
    };
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::StatusCode(status) => (status, "").into_response(),
            Self::Application(error) => application_error_response(error),
        }
    }
}

/// Strict same-origin guard for state-changing routes, mirroring
/// identity's router: a request whose Origin header is present and
/// mismatched is rejected; a same-origin browser request (no Origin
/// header, cookies sent first-party) passes.
pub fn ensure_browser_origin(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let origin = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if origin.is_empty() || origin.eq_ignore_ascii_case(expected) {
        return Ok(());
    }
    Err(ApiError::StatusCode(StatusCode::FORBIDDEN))
}
