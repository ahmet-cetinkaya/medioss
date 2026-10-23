use std::sync::Arc;

use super::record_position::RecordPosition;
use super::record_position::MAX_POSITION_MS;
use super::record_position_handler::RecordPositionHandler;
use crate::ports::{MeditationRepository, PlaybackStateRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct RecordPositionService {
    meditations: Arc<dyn MeditationRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
}

impl RecordPositionService {
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
impl RecordPositionHandler for RecordPositionService {
    async fn handle(&self, command: RecordPosition) -> Result<(), ApplicationError> {
        // Records the caller's position in media time (spec FR-015).
        let RecordPosition {
            user_id,
            meditation_id,
            position_ms,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        if !(0..=MAX_POSITION_MS).contains(&position_ms) {
            return Err(ApplicationError::InvalidInput(
                "position must be between 0 and 30 days in milliseconds",
            ));
        }
        self.playback
            .upsert_position(user_id, meditation_id, position_ms)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "record_position_service.tests.rs"]
mod tests;
