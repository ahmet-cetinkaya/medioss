use crate::ports::{AudioFormat, AudioValidator, SniffError};

/// Accepts any leading bytes that start with a known magic; rejects the rest.
/// Magic rules mirror the production sniffers and are the test oracle for
/// format detection without ffmpeg.
pub struct FakeValidator;

#[async_trait::async_trait]
impl AudioValidator for FakeValidator {
    async fn sniff(&self, leading: &[u8]) -> Result<AudioFormat, SniffError> {
        sniff_audio(leading)
    }
}

/// Production-shaped audio sniffing, shared by the fake and tests.
pub fn sniff_audio(bytes: &[u8]) -> Result<AudioFormat, SniffError> {
    if bytes.len() < 4 {
        return Err(SniffError::TooShort);
    }
    if bytes.starts_with(b"OggS") {
        let head = &bytes[..bytes.len().min(64)];
        return if head.windows(8).any(|window| window == b"OpusHead") {
            Ok(AudioFormat::OggOpus)
        } else {
            Ok(AudioFormat::OggVorbis)
        };
    }
    if bytes.starts_with(b"ID3") {
        return Ok(AudioFormat::Mp3);
    }
    if bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0 {
        return Ok(AudioFormat::Mp3);
    }
    if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WAVE" {
        return Ok(AudioFormat::Wav);
    }
    Err(SniffError::Unsupported)
}
