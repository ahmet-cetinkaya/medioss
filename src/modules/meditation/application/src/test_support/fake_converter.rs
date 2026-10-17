use crate::ports::AudioFormat;

/// Passes Ogg Opus through unchanged; rewrites other formats to a fake
/// "encoded" payload so tests can observe the conversion call.
#[derive(Clone)]
pub struct FakeConverter {
    /// When set, every conversion fails (simulates a decode error).
    pub fail: bool,
}

#[async_trait::async_trait]
impl crate::ports::OpusConverter for FakeConverter {
    async fn convert(
        &self,
        bytes: Vec<u8>,
        format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, crate::ports::StoredAudio), crate::ports::ConvertError> {
        if self.fail {
            return Err(crate::ports::ConvertError::Decode);
        }
        if matches!(format, AudioFormat::OggOpus | AudioFormat::Mp3) {
            return Ok((
                bytes,
                60_000,
                crate::ports::StoredAudio {
                    extension: if format == AudioFormat::Mp3 {
                        "mp3"
                    } else {
                        "opus"
                    },
                    content_type: if format == AudioFormat::Mp3 {
                        "audio/mpeg"
                    } else {
                        "audio/ogg"
                    },
                },
            ));
        }
        let mut encoded = b"fLaC".to_vec();
        encoded.extend_from_slice(b"fake-flac-payload");
        Ok((
            encoded,
            60_000,
            crate::ports::StoredAudio {
                extension: "flac",
                content_type: "audio/flac",
            },
        ))
    }
}
