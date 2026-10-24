use async_trait::async_trait;

/// Why cover normalization failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CoverNormalizationError {
    #[error("cover payload is invalid")]
    InvalidPayload,
    #[error("cover output exceeds the allowed size")]
    OutputTooLarge,
    #[error("cover processing worker failed")]
    WorkerFailed,
    #[error("cover processing is busy")]
    Busy,
}

/// Validates and normalizes cover uploads before persistence.
#[async_trait]
pub trait CoverValidator: Send + Sync {
    /// Converts a JPEG, PNG, or WebP cover to a bounded WebP image.
    async fn normalize(&self, bytes: Vec<u8>) -> Result<Vec<u8>, CoverNormalizationError>;
}
