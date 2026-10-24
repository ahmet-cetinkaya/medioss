use async_trait::async_trait;

use crate::ports::media::{AudioFormat, OpusConverter, StoredAudio};

pub struct FakeOpusConverter(pub Result<(Vec<u8>, i64, StoredAudio), String>);

#[async_trait]
impl OpusConverter for FakeOpusConverter {
    async fn convert(
        &self,
        _bytes: Vec<u8>,
        _format: AudioFormat,
    ) -> Result<(Vec<u8>, i64, StoredAudio), String> {
        self.0.clone()
    }
}
