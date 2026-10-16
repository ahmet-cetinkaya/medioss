use axum::http::StatusCode;
use soundscape_application::ApplicationError;

pub(crate) fn status_for(error: ApplicationError) -> StatusCode {
    match error {
        ApplicationError::NotFound => StatusCode::NOT_FOUND,
        ApplicationError::Duplicate | ApplicationError::CategoryInUse => StatusCode::CONFLICT,
        ApplicationError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        ApplicationError::UnsupportedMedia => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApplicationError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ApplicationError::Busy => StatusCode::SERVICE_UNAVAILABLE,
        ApplicationError::Storage(_) | ApplicationError::Repository(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[cfg(test)]
#[path = "error_mapping.tests.rs"]
mod tests;
