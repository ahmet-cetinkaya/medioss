use async_trait::async_trait;

use super::audio_validator::AudioFormat;
/// Why a conversion failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ConvertError {
    #[error("the audio stream could not be decoded")]
    Decode,
    #[error("the encoder is unavailable on this host")]
    Unavailable,
}

/// The stored form the converter produced, so callers persist the right
/// extension and content type for the bytes they were handed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredAudio {
    pub extension: &'static str,
    pub content_type: &'static str,
}

/// Converts accepted uploads to their stored, lossless-preserving form:
/// compressed inputs pass through byte-identical and PCM inputs become
/// lossless FLAC.
#[async_trait]
pub trait OpusConverter: Send + Sync {
    /// Converts `bytes` to its stored form and reports that form plus the
    /// decoded duration, so per-meditation progress can be shown as a
    /// percentage instead of raw position. A missing encoder surfaces as
    /// [`ConvertError::Unavailable`] so the API can answer 503 instead of
    /// blaming the media (plan D2).
    async fn convert(
        &self,
        bytes: Vec<u8>,
        format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, StoredAudio), ConvertError>;
}
