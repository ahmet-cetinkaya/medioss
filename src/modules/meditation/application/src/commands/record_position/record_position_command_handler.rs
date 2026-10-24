use std::sync::Arc;

use super::record_position_command::RecordPositionCommand;
use super::record_position_command_validator::RecordPositionCommandValidator;
use crate::ports::{CommandHandler, MeditationRepository, PlaybackStateRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct RecordPositionCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    validator: RecordPositionCommandValidator,
}

impl RecordPositionCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        playback: Arc<dyn PlaybackStateRepository>,
    ) -> Self {
        Self {
            meditations,
            playback,
            validator: RecordPositionCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RecordPositionCommand> for RecordPositionCommandHandler {
    type Output = ();

    async fn handle(&self, command: RecordPositionCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;

        // Records the caller's position in media time (spec FR-015).
        let RecordPositionCommand {
            user_id,
            meditation_id,
            position_ms,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        self.playback
            .upsert_position(user_id, meditation_id, position_ms)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "record_position_command_handler.tests.rs"]
mod tests;
