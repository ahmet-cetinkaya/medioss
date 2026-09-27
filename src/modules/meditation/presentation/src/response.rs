//! Error mapping and the same-origin guard.

use axum::{
    extract::multipart::Field,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::ApplicationError;

/// Reads one multipart field's bytes, rejecting it once it exceeds `cap`
/// and surfacing any mid-stream read error as `400` instead of silently
/// treating a truncated body as a complete one.
pub async fn read_field_capped(field: &mut Field<'_>, cap: usize) -> Result<Vec<u8>, ApiError> {
    let mut buffer = Vec::new();
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                buffer.extend_from_slice(&chunk);
                if buffer.len() > cap {
                    return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                }
            }
            Ok(None) => return Ok(buffer),
            Err(_) => return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST)),
        }
    }
}

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
/// identity's router: a request is accepted only when its Origin header is
/// present and matches exactly; a missing or mismatched Origin is rejected
/// (fail closed).
pub fn ensure_browser_origin(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let origin = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if origin.eq_ignore_ascii_case(expected) {
        return Ok(());
    }
    Err(ApiError::StatusCode(StatusCode::FORBIDDEN))
}
