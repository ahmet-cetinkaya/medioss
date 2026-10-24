use std::sync::Arc;

use super::set_sound_icon::SetSoundIcon;
use super::set_sound_icon_handler::SetSoundIconHandler;
use crate::ports::{
    media::{BlobStore, ImageNormalizer},
    SoundRepository,
};
use crate::services::{normalize_icon, store_sound_icon};
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct SetSoundIconService {
    sounds: Arc<dyn SoundRepository>,
    store: Arc<dyn BlobStore>,
    image_normalizer: Arc<dyn ImageNormalizer>,
}

impl SetSoundIconService {
    pub fn new(
        sounds: Arc<dyn SoundRepository>,
        store: Arc<dyn BlobStore>,
        image_normalizer: Arc<dyn ImageNormalizer>,
    ) -> Self {
        Self {
            sounds,
            store,
            image_normalizer,
        }
    }
}

#[async_trait::async_trait]
impl SetSoundIconHandler for SetSoundIconService {
    async fn handle(&self, command: SetSoundIcon) -> Result<Sound, ApplicationError> {
        let bytes = normalize_icon(self.image_normalizer.as_ref(), command.bytes).await?;
        store_sound_icon(self.sounds.as_ref(), self.store.as_ref(), command.id, bytes).await
    }
}

#[cfg(test)]
#[path = "set_sound_icon_service.tests.rs"]
mod tests;
