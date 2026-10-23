use std::sync::Arc;

use super::clear_sound_icon::ClearSoundIcon;
use super::clear_sound_icon_handler::ClearSoundIconHandler;
use crate::ports::{media::BlobStore, SoundRepository};
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct ClearSoundIconService {
    sounds: Arc<dyn SoundRepository>,
    store: Arc<dyn BlobStore>,
}

impl ClearSoundIconService {
    pub fn new(sounds: Arc<dyn SoundRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self { sounds, store }
    }
}

#[async_trait::async_trait]
impl ClearSoundIconHandler for ClearSoundIconService {
    async fn handle(&self, command: ClearSoundIcon) -> Result<Sound, ApplicationError> {
        let previous = self
            .sounds
            .find(command.id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?
            .icon;
        let sound = self
            .sounds
            .set_icon(command.id, None)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        if let Some(previous) = previous {
            let _ = self.store.delete(&previous.storage_key).await;
        }
        Ok(sound)
    }
}
