use axum::http::StatusCode;
use soundscape_application::{ports::media::ImageNormalizationError, ApplicationError};

use super::status_for;

#[test]
fn normalizer_outcomes_map_to_documented_statuses() {
    let cases = [
        (
            ImageNormalizationError::InvalidPayload,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            ImageNormalizationError::OutputTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        (
            ImageNormalizationError::Busy,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (
            ImageNormalizationError::WorkerFailed,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ];
    for (normalizer_error, expected) in cases {
        assert_eq!(
            status_for(ApplicationError::from(normalizer_error)),
            expected
        );
    }
}

#[test]
fn storage_failures_answer_internal_server_error() {
    let error = ApplicationError::Storage("disk".to_owned());
    assert_eq!(status_for(error), StatusCode::INTERNAL_SERVER_ERROR);
}
