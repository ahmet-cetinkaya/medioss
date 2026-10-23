use std::sync::Arc;

use super::set_completed::SetCompleted;
use super::set_completed_handler::SetCompletedHandler;
use crate::ports::{MeditationRepository, PlaybackStateRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct SetCompletedService {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
}

impl SetCompletedService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        playback: Arc<dyn PlaybackStateRepository>,
    ) -> Self {
        Self {
            meditations,
            playback,
        }
    }
}

#[async_trait::async_trait]
impl SetCompletedHandler for SetCompletedService {
    async fn handle(&self, command: SetCompleted) -> Result<(), ApplicationError> {
        // Never touches sessions or the stored position (spec FR-022).
        let SetCompleted {
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
#[path = "set_completed_service.tests.rs"]
mod tests;
