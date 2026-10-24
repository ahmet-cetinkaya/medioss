use super::*;
use crate::ports::media::ImageNormalizationError;
use crate::test_support::FakeImageNormalizer;

const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

#[tokio::test]
async fn valid_image_is_passed_to_the_normalizer() {
    let normalizer = FakeImageNormalizer(Ok(vec![7, 7]));
    let bytes = normalize_icon(&normalizer, PNG_MAGIC.to_vec())
        .await
        .unwrap();
    assert_eq!(bytes, vec![7, 7]);
}

#[tokio::test]
async fn icon_at_the_size_limit_is_accepted() {
    let mut bytes = PNG_MAGIC.to_vec();
    bytes.resize(MAX_ICON_BYTES, 0);
    let normalizer = FakeImageNormalizer(Ok(vec![1]));
    assert!(normalize_icon(&normalizer, bytes).await.is_ok());
}

#[tokio::test]
async fn oversized_icon_is_rejected() {
    let normalizer = FakeImageNormalizer(Ok(vec![1]));
    let result = normalize_icon(&normalizer, vec![0; MAX_ICON_BYTES + 1]).await;
    assert_eq!(result, Err(ApplicationError::PayloadTooLarge));
}

#[tokio::test]
async fn unrecognised_bytes_are_unsupported_media() {
    let normalizer = FakeImageNormalizer(Ok(vec![1]));
    let result = normalize_icon(&normalizer, b"<svg/>".to_vec()).await;
    assert_eq!(result, Err(ApplicationError::UnsupportedMedia));
}

#[tokio::test]
async fn normalizer_errors_are_mapped() {
    let normalizer = FakeImageNormalizer(Err(ImageNormalizationError::OutputTooLarge));
    let result = normalize_icon(&normalizer, PNG_MAGIC.to_vec()).await;
    assert_eq!(result, Err(ApplicationError::PayloadTooLarge));
}
