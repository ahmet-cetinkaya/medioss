use crate::ports::{AudioFormat, StagedMedia};

/// Reports the stored form for each format and reuses the staged file as the
/// converted output.
#[derive(Clone)]
pub struct FakeConverter {
    /// When set, every conversion fails (simulates a decode error).
    pub fail: bool,
}

#[async_trait::async_trait]
impl crate::ports::OpusConverter for FakeConverter {
    async fn convert(
        &self,
        media: StagedMedia,
        format: AudioFormat,
    ) -> Result<(StagedMedia, i64, crate::ports::StoredAudio), crate::ports::ConvertError> {
        if self.fail {
            return Err(crate::ports::ConvertError::Decode);
        }
        if matches!(format, AudioFormat::OggOpus | AudioFormat::Mp3) {
            return Ok((
                media,
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
        Ok((
            media,
            60_000,
            crate::ports::StoredAudio {
                extension: "flac",
                content_type: "audio/flac",
            },
        ))
    }
}
