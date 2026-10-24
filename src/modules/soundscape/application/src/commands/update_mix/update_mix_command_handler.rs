use std::sync::Arc;

use super::update_mix_command::UpdateMixCommand;
use super::update_mix_command_validator::UpdateMixCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{Clock, MixRepository, NewMix, SoundRepository};
use crate::services::ensure_sounds_exist;
use crate::ApplicationError;
use soundscape_domain::Mix;

pub struct UpdateMixCommandHandler {
    mixes: Arc<dyn MixRepository>,
    sounds: Arc<dyn SoundRepository>,
    clock: Arc<dyn Clock>,
    validator: UpdateMixCommandValidator,
}

impl UpdateMixCommandHandler {
    pub fn new(
        mixes: Arc<dyn MixRepository>,
        sounds: Arc<dyn SoundRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            mixes,
            sounds,
            clock,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UpdateMixCommand> for UpdateMixCommandHandler {
    type Output = Mix;

    async fn handle(&self, command: UpdateMixCommand) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&command)?;
        ensure_sounds_exist(self.sounds.as_ref(), &command.items).await?;
        self.mixes
            .replace(
                command.id,
                NewMix {
                    user_id: command.user_id,
                    name: self.validator.normalized_name(&command),
                    items: command.items,
                    now: self.clock.now().await,
                },
            )
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "update_mix_command_handler.tests.rs"]
mod tests;
