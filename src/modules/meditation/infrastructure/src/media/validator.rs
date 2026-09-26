//! Pure-Rust magic-byte sniffing; no subprocess.
use async_trait::async_trait;
use meditation_application::ports::{AudioFormat, AudioValidator, SniffError};

/// Sniffs audio formats from leading bytes (ID3 / MPEG sync / RIFF-WAVE /
/// OggS with OpusHead detection in the first page).
#[derive(Clone, Copy, Default)]
pub struct FfmpegAudioValidator;

#[async_trait]
impl AudioValidator for FfmpegAudioValidator {
    async fn sniff(&self, leading: &[u8]) -> Result<AudioFormat, SniffError> {
        if leading.len() < 4 {
            return Err(SniffError::TooShort);
        }
        if leading.starts_with(b"OggS") {
            let head = &leading[..leading.len().min(64)];
            return if head.windows(8).any(|window| window == b"OpusHead") {
                Ok(AudioFormat::OggOpus)
            } else {
                Ok(AudioFormat::OggVorbis)
            };
        }
        if leading.starts_with(b"ID3") {
            return Ok(AudioFormat::Mp3);
        }
        if leading[0] == 0xFF && leading[1] & 0xE0 == 0xE0 {
            return Ok(AudioFormat::Mp3);
        }
        if leading.starts_with(b"RIFF") && leading.len() >= 12 && &leading[8..12] == b"WAVE" {
            return Ok(AudioFormat::Wav);
        }
        Err(SniffError::Unsupported)
    }
}

#[cfg(test)]
#[path = "validator.tests.rs"]
mod tests;
