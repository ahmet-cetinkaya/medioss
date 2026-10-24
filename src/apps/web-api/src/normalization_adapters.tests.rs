use super::*;

use identity_application::ports::AvatarStoreError;
use meditation_application::ports::CoverNormalizationError;
use soundscape_application::ports::media::ImageNormalizationError as SoundscapeError;

#[test]
fn identity_avatar_store_error_maps_every_variant() {
    assert_eq!(
        AvatarStoreError::from(ImageNormalizationError::InvalidPayload),
        AvatarStoreError::InvalidPayload
    );
    assert_eq!(
        AvatarStoreError::from(ImageNormalizationError::OutputTooLarge),
        AvatarStoreError::OutputTooLarge
    );
    assert_eq!(
        AvatarStoreError::from(ImageNormalizationError::WorkerFailed),
        AvatarStoreError::WorkerFailed
    );
    assert_eq!(
        AvatarStoreError::from(ImageNormalizationError::Busy),
        AvatarStoreError::Busy
    );
}

#[test]
fn meditation_cover_error_maps_every_variant() {
    assert_eq!(
        CoverNormalizationError::from(ImageNormalizationError::InvalidPayload),
        CoverNormalizationError::InvalidPayload
    );
    assert_eq!(
        CoverNormalizationError::from(ImageNormalizationError::OutputTooLarge),
        CoverNormalizationError::OutputTooLarge
    );
    assert_eq!(
        CoverNormalizationError::from(ImageNormalizationError::WorkerFailed),
        CoverNormalizationError::WorkerFailed
    );
    assert_eq!(
        CoverNormalizationError::from(ImageNormalizationError::Busy),
        CoverNormalizationError::Busy
    );
}

#[test]
fn soundscape_media_error_maps_every_variant() {
    assert_eq!(
        SoundscapeError::from(ImageNormalizationError::InvalidPayload),
        SoundscapeError::InvalidPayload
    );
    assert_eq!(
        SoundscapeError::from(ImageNormalizationError::OutputTooLarge),
        SoundscapeError::OutputTooLarge
    );
    assert_eq!(
        SoundscapeError::from(ImageNormalizationError::WorkerFailed),
        SoundscapeError::WorkerFailed
    );
    assert_eq!(
        SoundscapeError::from(ImageNormalizationError::Busy),
        SoundscapeError::Busy
    );
}
