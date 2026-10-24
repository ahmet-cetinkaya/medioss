use async_trait::async_trait;

/// Byte formats the upload pipeline accepts, detected from leading bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// MPEG audio layer III, with or without an ID3v2 tag.
    Mp3,
    /// RIFF/WAVE PCM audio.
    Wav,
    /// Ogg container already carrying Opus: stored as-is.
    OggOpus,
    /// Ogg container carrying Vorbis: re-encoded to Opus.
    OggVorbis,
}

/// Why byte sniffing failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SniffError {
    #[error("not a supported audio format")]
    Unsupported,
    #[error("too few bytes to identify the format")]
    TooShort,
}

/// Detects the audio format from leading bytes (magic sniffing only).
#[async_trait]
pub trait AudioValidator: Send + Sync {
    /// Sniffs the format from the leading bytes of an upload.
    async fn sniff(&self, leading: &[u8]) -> Result<AudioFormat, SniffError>;
}
