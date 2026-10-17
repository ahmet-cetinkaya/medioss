use axum::http::StatusCode;
use identity_application::ports::AvatarStoreError;
use identity_application::ApplicationError;
use identity_domain::IdentityError;

use super::status_for;

#[test]
fn avatar_normalizer_outcomes_map_to_documented_statuses() {
    let cases = [
        (
            AvatarStoreError::InvalidPayload,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            AvatarStoreError::OutputTooLarge,
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
        (AvatarStoreError::Busy, StatusCode::SERVICE_UNAVAILABLE),
        (
            AvatarStoreError::WorkerFailed,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (AvatarStoreError::Storage, StatusCode::INTERNAL_SERVER_ERROR),
    ];
    for (store_error, expected) in cases {
        assert_eq!(status_for(&ApplicationError::from(store_error)), expected);
    }
}

#[test]
fn bad_profile_fields_stay_bad_request() {
    let error = ApplicationError::InvalidInput(IdentityError::InvalidProfile);
    assert_eq!(status_for(&error), StatusCode::BAD_REQUEST);
}
