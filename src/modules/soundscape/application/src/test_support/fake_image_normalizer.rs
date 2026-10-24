use async_trait::async_trait;

use crate::ports::media::{ImageNormalizationError, ImageNormalizer};

pub struct FakeImageNormalizer(pub Result<Vec<u8>, ImageNormalizationError>);

#[async_trait]
impl ImageNormalizer for FakeImageNormalizer {
    async fn normalize(&self, _bytes: Vec<u8>) -> Result<Vec<u8>, ImageNormalizationError> {
        self.0.clone()
    }
}
