use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use identity_application::ApplicationError;
use identity_domain::IdentityError;

pub struct ApiError(pub(crate) ApplicationError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, retry_after): (StatusCode, String, _) = match &self.0 {
            ApplicationError::InvalidInput(IdentityError::LoginThrottled { retry_after }) => (
                StatusCode::TOO_MANY_REQUESTS,
                "too many attempts".to_owned(),
                Some(retry_after.as_secs()),
            ),
            // Domain messages are user-facing by design and contain no secrets.
            ApplicationError::InvalidInput(error) => {
                (StatusCode::BAD_REQUEST, error.to_string(), None)
            }
            ApplicationError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid credentials".to_owned(),
                None,
            ),
            ApplicationError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_owned(), None),
            ApplicationError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload too large".to_owned(),
                None,
            ),
            ApplicationError::LastActiveAdmin => (
                StatusCode::BAD_REQUEST,
                "operation would leave the instance without an active admin".to_owned(),
                None,
            ),
            ApplicationError::SetupAlreadyCompleted => (
                StatusCode::CONFLICT,
                "setup already completed".to_owned(),
                None,
            ),
            ApplicationError::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "email already exists".to_owned(),
                None,
            ),
            ApplicationError::UserNotFound => (
                StatusCode::UNAUTHORIZED,
                "invalid credentials".to_owned(),
                None,
            ),
            ApplicationError::ProcessingUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "avatar processing is temporarily unavailable".to_owned(),
                None,
            ),
            ApplicationError::PasswordHasher | ApplicationError::Repository => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_owned(),
                None,
            ),
        };
        let mut response = (
            status,
            Json(serde_json::json!({
                "error": message,
                "retry_after": retry_after,
            })),
        )
            .into_response();
        if let Some(secs) = retry_after {
            // At least one second: sub-second penalties round up so the
            // client never sees retry_after = 0.
            let headers = response.headers_mut();
            headers.insert(
                "Retry-After",
                secs.max(1)
                    .to_string()
                    .parse()
                    .expect("retry-after is a valid header value"),
            );
        }
        response
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        Self(error)
    }
}

impl From<IdentityError> for ApiError {
    fn from(error: IdentityError) -> Self {
        Self(ApplicationError::InvalidInput(error))
    }
}

impl From<tower_sessions::session::Error> for ApiError {
    fn from(_: tower_sessions::session::Error) -> Self {
        Self(ApplicationError::Repository)
    }
}

#[cfg(test)]
impl ApiError {
    pub(crate) fn into_response_status(self) -> StatusCode {
        use axum::response::IntoResponse;
        let (parts, _) = self.into_response().into_parts();
        parts.status
    }
}
