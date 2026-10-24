use std::sync::Arc;

use super::create_mix_command::CreateMixCommand;
use super::create_mix_command_validator::CreateMixCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{Clock, MixRepository, NewMix, SoundRepository};
use crate::services::ensure_sounds_exist;
use crate::ApplicationError;
use soundscape_domain::Mix;

pub struct CreateMixCommandHandler {
    mixes: Arc<dyn MixRepository>,
    sounds: Arc<dyn SoundRepository>,
    clock: Arc<dyn Clock>,
    validator: CreateMixCommandValidator,
}

impl CreateMixCommandHandler {
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
impl CommandHandler<CreateMixCommand> for CreateMixCommandHandler {
    type Output = Mix;

    async fn handle(&self, command: CreateMixCommand) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&command)?;
        ensure_sounds_exist(self.sounds.as_ref(), &command.items).await?;
        self.mixes
            .create(NewMix {
                user_id: command.user_id,
                name: self.validator.normalized_name(&command),
                items: command.items,
                now: self.clock.now().await,
            })
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "create_mix_command_handler.tests.rs"]
mod tests;
