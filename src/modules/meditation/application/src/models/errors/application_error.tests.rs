use super::{ApplicationError, COVER_PROCESSING_FAILED};
use crate::ports::CoverNormalizationError;

#[test]
fn cover_errors_map_to_application_errors() {
    assert_eq!(
        ApplicationError::from(CoverNormalizationError::InvalidPayload),
        ApplicationError::UnsupportedMedia
    );
    assert_eq!(
        ApplicationError::from(CoverNormalizationError::OutputTooLarge),
        ApplicationError::PayloadTooLarge
    );
    assert_eq!(
        ApplicationError::from(CoverNormalizationError::WorkerFailed),
        ApplicationError::Storage(COVER_PROCESSING_FAILED.to_owned())
    );
    assert_eq!(
        ApplicationError::from(CoverNormalizationError::Busy),
        ApplicationError::Busy
    );
}

#[test]
fn messages_describe_the_failure() {
    assert_eq!(
        ApplicationError::InvalidInput("name is empty").to_string(),
        "input is invalid: name is empty"
    );
    assert_eq!(
        ApplicationError::Repository("db down".into()).to_string(),
        "repository operation failed: db down"
    );
    assert_eq!(
        ApplicationError::NotFound.to_string(),
        "the requested resource was not found"
    );
}
