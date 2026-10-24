use std::sync::Arc;

use super::get_playback_state_query::GetPlaybackStateQuery;
use super::get_playback_state_query_validator::GetPlaybackStateQueryValidator;
use crate::ports::PlaybackState;
use crate::ports::{MeditationRepository, PlaybackStateRepository, QueryHandler};
use crate::services::load_meditation;
use crate::ApplicationError;

pub struct GetPlaybackStateQueryHandler {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    validator: GetPlaybackStateQueryValidator,
}

impl GetPlaybackStateQueryHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        playback: Arc<dyn PlaybackStateRepository>,
    ) -> Self {
        Self {
            meditations,
            playback,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<GetPlaybackStateQuery> for GetPlaybackStateQueryHandler {
    type Output = PlaybackState;

    async fn handle(
        &self,
        query: GetPlaybackStateQuery,
    ) -> Result<PlaybackState, ApplicationError> {
        self.validator.validate(&query)?;
        // Zero/false when never played (spec FR-016).
        let GetPlaybackStateQuery {
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
#[path = "get_playback_state_query_handler.tests.rs"]
mod tests;
