use async_trait::async_trait;

use crate::ports::media::{AudioFormat, AudioValidator};

pub struct FakeAudioValidator(pub Result<AudioFormat, ()>);

#[async_trait]
impl AudioValidator for FakeAudioValidator {
    async fn sniff(&self, _bytes: &[u8]) -> Result<AudioFormat, ()> {
        self.0
    }
}
