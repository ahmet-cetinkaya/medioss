use std::sync::Arc;

use super::set_sound_icon_command::SetSoundIconCommand;
use super::set_sound_icon_command_validator::SetSoundIconCommandValidator;
use crate::ports::{
    media::{BlobStore, ImageNormalizer},
    CommandHandler, SoundRepository,
};
use crate::services::{normalize_icon, store_sound_icon};
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct SetSoundIconCommandHandler {
    sounds: Arc<dyn SoundRepository>,
    store: Arc<dyn BlobStore>,
    image_normalizer: Arc<dyn ImageNormalizer>,
    validator: SetSoundIconCommandValidator,
}

impl SetSoundIconCommandHandler {
    pub fn new(
        sounds: Arc<dyn SoundRepository>,
        store: Arc<dyn BlobStore>,
        image_normalizer: Arc<dyn ImageNormalizer>,
    ) -> Self {
        Self {
            sounds,
            store,
            image_normalizer,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<SetSoundIconCommand> for SetSoundIconCommandHandler {
    type Output = Sound;

    async fn handle(&self, command: SetSoundIconCommand) -> Result<Sound, ApplicationError> {
        self.validator.validate(&command)?;
        let bytes = normalize_icon(self.image_normalizer.as_ref(), command.bytes).await?;
        store_sound_icon(self.sounds.as_ref(), self.store.as_ref(), command.id, bytes).await
    }
}

#[cfg(test)]
#[path = "set_sound_icon_command_handler.tests.rs"]
mod tests;
