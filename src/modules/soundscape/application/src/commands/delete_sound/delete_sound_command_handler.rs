use std::sync::Arc;

use super::delete_sound_command::DeleteSoundCommand;
use super::delete_sound_command_validator::DeleteSoundCommandValidator;
use crate::ports::{media::BlobStore, CommandHandler, SoundRepository};
use crate::services::remove_sound;
use crate::ApplicationError;

pub struct DeleteSoundCommandHandler {
    sounds: Arc<dyn SoundRepository>,
    store: Arc<dyn BlobStore>,
    validator: DeleteSoundCommandValidator,
}

impl DeleteSoundCommandHandler {
    pub fn new(sounds: Arc<dyn SoundRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self {
            sounds,
            store,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteSoundCommand> for DeleteSoundCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteSoundCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        remove_sound(self.sounds.as_ref(), self.store.as_ref(), command.id).await
    }
}

#[cfg(test)]
#[path = "delete_sound_command_handler.tests.rs"]
mod tests;
