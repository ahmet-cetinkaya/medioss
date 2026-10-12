use soundscape_application::ports::media::ImageNormalizer as _;

use super::*;

#[tokio::test(flavor = "current_thread")]
async fn icon_normalizer_maps_core_errors_to_the_ports_typed_error() {
    let _lock = crate::image_normalization::NORMALIZATION_TEST_LOCK
        .lock()
        .await;

    // Before this mapping was typed, every failure collapsed to `()`; the
    // adapter must hand the application the exact core variant.
    let error = IconNormalizer
        .normalize(b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec())
        .await
        .unwrap_err();
    assert_eq!(
        error,
        soundscape_application::ports::media::ImageNormalizationError::InvalidPayload
    );

    let error = IconNormalizer
        .normalize(b"plain text".to_vec())
        .await
        .unwrap_err();
    assert_eq!(
        error,
        soundscape_application::ports::media::ImageNormalizationError::InvalidPayload
    );
}
