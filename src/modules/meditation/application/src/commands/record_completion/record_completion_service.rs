use std::sync::Arc;

use super::record_completion::RecordCompletion;
use super::record_completion_handler::RecordCompletionHandler;
use crate::ports::MeditationSession;
use crate::ports::{MeditationRepository, PlaybackStateRepository, SessionRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct RecordCompletionService {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    sessions: Arc<dyn SessionRepository>,
}

impl RecordCompletionService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        playback: Arc<dyn PlaybackStateRepository>,
        sessions: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            meditations,
            playback,
            sessions,
        }
    }
}

#[async_trait::async_trait]
impl RecordCompletionHandler for RecordCompletionService {
    async fn handle(
        &self,
        command: RecordCompletion,
    ) -> Result<MeditationSession, ApplicationError> {
        // Idempotent per (user, meditation, started_at): a retry returns the
        // same session, flips `completed` to true, and resets the position
        // (spec FR-019..021).
        let RecordCompletion {
            user_id,
            meditation_id,
            started_at,
            finished_at,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        if finished_at < started_at {
            return Err(ApplicationError::InvalidInput(
                "finished_at must not precede started_at",
            ));
        }
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
#[path = "record_completion_service.tests.rs"]
mod tests;
