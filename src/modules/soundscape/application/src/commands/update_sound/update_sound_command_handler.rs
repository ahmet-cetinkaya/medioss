use std::sync::Arc;

use super::update_sound_command::UpdateSoundCommand;
use super::update_sound_command_validator::UpdateSoundCommandValidator;
use crate::ports::{CategoryRepository, CommandHandler, SoundRepository};
use crate::services::{ensure_category_exists, parse_icon_key};
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct UpdateSoundCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    sounds: Arc<dyn SoundRepository>,
    validator: UpdateSoundCommandValidator,
}

impl UpdateSoundCommandHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>, sounds: Arc<dyn SoundRepository>) -> Self {
        Self {
            categories,
            sounds,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UpdateSoundCommand> for UpdateSoundCommandHandler {
    type Output = Sound;

    async fn handle(&self, command: UpdateSoundCommand) -> Result<Sound, ApplicationError> {
        self.validator.validate(&command)?;
        let icon_key = parse_icon_key(command.icon_key.as_deref())?;
        ensure_category_exists(self.categories.as_ref(), command.category_id).await?;
        self.sounds
            .update(
                command.id,
                &command.name,
                command.category_id,
                icon_key.as_deref(),
            )
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "update_sound_command_handler.tests.rs"]
mod tests;
