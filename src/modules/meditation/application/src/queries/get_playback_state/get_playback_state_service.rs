use std::sync::Arc;

use super::get_playback_state::GetPlaybackState;
use super::get_playback_state_handler::GetPlaybackStateHandler;
use crate::ports::PlaybackState;
use crate::ports::{MeditationRepository, PlaybackStateRepository};
use crate::services::load_meditation;
use crate::ApplicationError;

pub struct GetPlaybackStateService {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
}

impl GetPlaybackStateService {
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
impl GetPlaybackStateHandler for GetPlaybackStateService {
    async fn handle(&self, query: GetPlaybackState) -> Result<PlaybackState, ApplicationError> {
        // Zero/false when never played (spec FR-016).
        let GetPlaybackState {
            user_id,
            meditation_id,
        } = query;
        load_meditation(self.meditations.as_ref(), meditation_id).await?;
        Ok(self
            .playback
            .get(user_id, meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
            .unwrap_or(PlaybackState {
                position_ms: 0,
                completed: false,
            }))
    }
}

#[cfg(test)]
#[path = "get_playback_state_service.tests.rs"]
mod tests;
