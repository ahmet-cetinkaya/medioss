use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use mood_tracker_application::ApplicationError;

pub enum ApiError {
    StatusCode(StatusCode),
    Application(ApplicationError),
}
impl ApiError {
    pub fn unauthorized() -> Self {
        Self::StatusCode(StatusCode::UNAUTHORIZED)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::StatusCode(status) => status.into_response(),
            Self::Application(error) => {
                let status = match error {
                    ApplicationError::InvalidInput(_) => StatusCode::BAD_REQUEST,
                    ApplicationError::NotFound => StatusCode::NOT_FOUND,
                    ApplicationError::Repository(_) => StatusCode::SERVICE_UNAVAILABLE,
                };
                (
                    status,
                    Json(serde_json::json!({"error": error.to_string()})),
                )
                    .into_response()
            }
        }
    }
}
pub fn ensure_browser_origin(headers: &HeaderMap, origin: &str) -> Result<(), ApiError> {
    // Fail closed: a missing Origin is not proven to be a same-origin browser
    // request, so reject it like a mismatched one. Compare case-insensitively;
    // scheme and host are ASCII case-insensitive per RFC 3986.
    headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.eq_ignore_ascii_case(origin))
        .ok_or(ApiError::StatusCode(StatusCode::FORBIDDEN))
        .map(|_| ())
}
