use std::sync::Arc;

use super::record_completion_command::RecordCompletionCommand;
use super::record_completion_command_validator::RecordCompletionCommandValidator;
use crate::ports::{
    CommandHandler, MeditationRepository, MeditationSession, PlaybackStateRepository,
    SessionRepository,
};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct RecordCompletionCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    sessions: Arc<dyn SessionRepository>,
    validator: RecordCompletionCommandValidator,
}

impl RecordCompletionCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        playback: Arc<dyn PlaybackStateRepository>,
        sessions: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            meditations,
            playback,
            sessions,
            validator: RecordCompletionCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RecordCompletionCommand> for RecordCompletionCommandHandler {
    type Output = MeditationSession;

    async fn handle(
        &self,
        command: RecordCompletionCommand,
    ) -> Result<MeditationSession, ApplicationError> {
        self.validator.validate(&command)?;

        // Idempotent per (user, meditation, started_at): a retry returns the
        // same session, flips `completed` to true, and resets the position
        // (spec FR-019..021).
        let RecordCompletionCommand {
            user_id,
            meditation_id,
            started_at,
            finished_at,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        let session = self
            .sessions
            .record_if_new(user_id, meditation_id, started_at, finished_at)
            .await
            .map_err(ApplicationError::Repository)?;
        self.playback
            .set_completed(user_id, meditation_id, true)
            .await
            .map_err(ApplicationError::Repository)?;
        self.playback
            .reset_position(user_id, meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
        Ok(session)
    }
}

#[cfg(test)]
#[path = "record_completion_command_handler.tests.rs"]
mod tests;
