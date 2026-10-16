use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use identity_application::ApplicationError;
use identity_domain::IdentityError;

pub struct ApiError(pub(crate) ApplicationError);

/// The one place that decides which HTTP status each application outcome
/// (including every avatar normalizer outcome) answers with.
pub(crate) fn status_for(error: &ApplicationError) -> StatusCode {
    match error {
        ApplicationError::InvalidInput(IdentityError::LoginThrottled { .. }) => {
            StatusCode::TOO_MANY_REQUESTS
        }
        ApplicationError::InvalidInput(_) | ApplicationError::LastActiveAdmin => {
            StatusCode::BAD_REQUEST
        }
        ApplicationError::InvalidCredentials | ApplicationError::UserNotFound => {
            StatusCode::UNAUTHORIZED
        }
        ApplicationError::Forbidden => StatusCode::FORBIDDEN,
        ApplicationError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ApplicationError::UnsupportedMedia => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApplicationError::SetupAlreadyCompleted | ApplicationError::EmailAlreadyExists => {
            StatusCode::CONFLICT
        }
        ApplicationError::ProcessingUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        ApplicationError::PasswordHasher | ApplicationError::Repository => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn message_for(error: &ApplicationError) -> String {
    match error {
        ApplicationError::InvalidInput(IdentityError::LoginThrottled { .. }) => {
            "too many attempts".to_owned()
        }
        // Domain messages are user-facing by design and contain no secrets.
        ApplicationError::InvalidInput(error) => error.to_string(),
        ApplicationError::InvalidCredentials | ApplicationError::UserNotFound => {
            "invalid credentials".to_owned()
        }
        ApplicationError::Forbidden => "forbidden".to_owned(),
        ApplicationError::PayloadTooLarge => "payload too large".to_owned(),
        ApplicationError::UnsupportedMedia => "unsupported media type".to_owned(),
        ApplicationError::LastActiveAdmin => {
            "operation would leave the instance without an active admin".to_owned()
        }
        ApplicationError::SetupAlreadyCompleted => "setup already completed".to_owned(),
        ApplicationError::EmailAlreadyExists => "email already exists".to_owned(),
        ApplicationError::ProcessingUnavailable => {
            "avatar processing is temporarily unavailable".to_owned()
        }
        ApplicationError::PasswordHasher | ApplicationError::Repository => {
            "internal error".to_owned()
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = status_for(&self.0);
        let message = message_for(&self.0);
        let retry_after = match &self.0 {
            ApplicationError::InvalidInput(IdentityError::LoginThrottled { retry_after }) => {
                Some(retry_after.as_secs())
            }
            _ => None,
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

#[cfg(test)]
#[path = "api_error.tests.rs"]
mod tests;
