use std::sync::Arc;

use super::clear_sound_icon_command::ClearSoundIconCommand;
use super::clear_sound_icon_command_validator::ClearSoundIconCommandValidator;
use crate::ports::{media::BlobStore, CommandHandler, SoundRepository};
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct ClearSoundIconCommandHandler {
    sounds: Arc<dyn SoundRepository>,
    store: Arc<dyn BlobStore>,
    validator: ClearSoundIconCommandValidator,
}

impl ClearSoundIconCommandHandler {
    pub fn new(sounds: Arc<dyn SoundRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self {
            sounds,
            store,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<ClearSoundIconCommand> for ClearSoundIconCommandHandler {
    type Output = Sound;

    async fn handle(&self, command: ClearSoundIconCommand) -> Result<Sound, ApplicationError> {
        self.validator.validate(&command)?;
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

#[cfg(test)]
#[path = "clear_sound_icon_command_handler.tests.rs"]
mod tests;
