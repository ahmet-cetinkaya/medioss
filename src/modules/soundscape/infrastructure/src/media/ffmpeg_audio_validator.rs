use async_trait::async_trait;
use soundscape_application::ports::media::{AudioFormat, AudioValidator};

pub struct FfmpegAudioValidator;
#[async_trait]
impl AudioValidator for FfmpegAudioValidator {
    async fn sniff(&self, bytes: &[u8]) -> Result<AudioFormat, ()> {
        if bytes.starts_with(b"OggS") {
            return if bytes[..bytes.len().min(64)]
                .windows(8)
                .any(|x| x == b"OpusHead")
            {
                Ok(AudioFormat::OggOpus)
            } else {
                Ok(AudioFormat::OggVorbis)
            };
        }
        if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WAVE" {
            return Ok(AudioFormat::Wav);
        }
        if bytes.starts_with(b"ID3")
            || (bytes.len() > 1 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0)
        {
            return Ok(AudioFormat::Mp3);
        }
        Err(())
    }
}
