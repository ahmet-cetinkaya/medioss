use std::sync::Arc;

use super::set_completed_command::SetCompletedCommand;
use super::set_completed_command_validator::SetCompletedCommandValidator;
use crate::ports::{CommandHandler, MeditationRepository, PlaybackStateRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct SetCompletedCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    validator: SetCompletedCommandValidator,
}

impl SetCompletedCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        playback: Arc<dyn PlaybackStateRepository>,
    ) -> Self {
        Self {
            meditations,
            playback,
            validator: SetCompletedCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<SetCompletedCommand> for SetCompletedCommandHandler {
    type Output = ();

    async fn handle(&self, command: SetCompletedCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;

        // Never touches sessions or the stored position (spec FR-022).
        let SetCompletedCommand {
            user_id,
            meditation_id,
            completed,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        self.playback
            .set_completed(user_id, meditation_id, completed)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "set_completed_command_handler.tests.rs"]
mod tests;
