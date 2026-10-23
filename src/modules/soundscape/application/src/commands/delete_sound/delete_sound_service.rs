use std::sync::Arc;

use super::delete_sound::DeleteSound;
use super::delete_sound_handler::DeleteSoundHandler;
use crate::ports::{media::BlobStore, SoundRepository};
use crate::services::remove_sound;
use crate::ApplicationError;

pub struct DeleteSoundService {
    sounds: Arc<dyn SoundRepository>,
    store: Arc<dyn BlobStore>,
}

impl DeleteSoundService {
    pub fn new(sounds: Arc<dyn SoundRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self { sounds, store }
    }
}

#[async_trait::async_trait]
impl DeleteSoundHandler for DeleteSoundService {
    async fn handle(&self, command: DeleteSound) -> Result<(), ApplicationError> {
        remove_sound(self.sounds.as_ref(), self.store.as_ref(), command.id).await
    }
}
