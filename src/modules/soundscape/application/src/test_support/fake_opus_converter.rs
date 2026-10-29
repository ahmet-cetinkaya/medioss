use async_trait::async_trait;

use super::FakeUploadStaging;
use crate::ports::media::{AudioFormat, OpusConverter, StoredAudio};
use crate::ports::staged_upload::StagedMedia;

/// Fake converter: on success it stages the given output bytes as a new
/// staged file, like the real converter writes its output beside its input.
pub struct FakeOpusConverter {
    staging: FakeUploadStaging,
    outcome: Result<(Vec<u8>, i64, StoredAudio), String>,
}

impl FakeOpusConverter {
    pub fn new(
        staging: FakeUploadStaging,
        outcome: Result<(Vec<u8>, i64, StoredAudio), String>,
    ) -> Self {
        Self { staging, outcome }
    }
}

#[async_trait]
impl OpusConverter for FakeOpusConverter {
    async fn convert(
        &self,
        _media: StagedMedia,
        _format: AudioFormat,
    ) -> Result<(StagedMedia, i64, StoredAudio), String> {
        let (bytes, duration_ms, stored) = self.outcome.clone()?;
        Ok((self.staging.stage(&bytes), duration_ms, stored))
    }
}
