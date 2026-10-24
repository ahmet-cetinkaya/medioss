use super::*;

#[test]
fn unique_violation_maps_to_duplicate() {
    let error = ApplicationError::from_category_repository(
        "UNIQUE constraint failed: soundscape_categories.name".into(),
    );
    assert_eq!(error, ApplicationError::Duplicate);
}

#[test]
fn other_category_failures_stay_repository_errors() {
    let error = ApplicationError::from_category_repository("disk full".into());
    assert_eq!(error, ApplicationError::Repository("disk full".into()));
}

#[test]
fn image_normalization_errors_map_to_application_errors() {
    assert_eq!(
        ApplicationError::from(ImageNormalizationError::InvalidPayload),
        ApplicationError::UnsupportedMedia
    );
    assert_eq!(
        ApplicationError::from(ImageNormalizationError::OutputTooLarge),
        ApplicationError::PayloadTooLarge
    );
    assert_eq!(
        ApplicationError::from(ImageNormalizationError::WorkerFailed),
        ApplicationError::Storage(ICON_PROCESSING_FAILED.into())
    );
    assert_eq!(
        ApplicationError::from(ImageNormalizationError::Busy),
        ApplicationError::Busy
    );
}

#[test]
fn display_messages_carry_their_detail() {
    assert_eq!(
        ApplicationError::InvalidInput("mix name").to_string(),
        "input is invalid: mix name"
    );
    assert_eq!(
        ApplicationError::Repository("boom".into()).to_string(),
        "repository operation failed: boom"
    );
}
