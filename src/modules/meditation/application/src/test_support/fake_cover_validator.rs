use crate::ports::{CoverNormalizationError, CoverValidator};

/// Accepts PNG/JPEG/WebP magics.
pub struct FakeCoverValidator;

#[async_trait::async_trait]
impl CoverValidator for FakeCoverValidator {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<Vec<u8>, CoverNormalizationError> {
        if bytes.len() < 12 {
            return Err(CoverNormalizationError::InvalidPayload);
        }
        if bytes.starts_with(&[0x89, b'P', b'N', b'G'])
            || bytes.starts_with(&[0xFF, 0xD8, 0xFF])
            || (bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP")
        {
            return Ok(bytes);
        }
        Err(CoverNormalizationError::InvalidPayload)
    }
}

/// Always fails with the configured error, for mapping tests.
pub struct FailingCoverValidator(pub CoverNormalizationError);

#[async_trait::async_trait]
impl CoverValidator for FailingCoverValidator {
    async fn normalize(&self, _bytes: Vec<u8>) -> Result<Vec<u8>, CoverNormalizationError> {
        Err(self.0)
    }
}
