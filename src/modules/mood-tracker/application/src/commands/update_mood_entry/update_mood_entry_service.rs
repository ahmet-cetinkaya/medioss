use std::sync::Arc;

use super::update_mood_entry::UpdateMoodEntry;
use super::update_mood_entry_handler::UpdateMoodEntryHandler;
use crate::ports::{Clock, MoodEntryRepository};
use crate::services::normalize_note::normalize_note;
use crate::ApplicationError;
use mood_tracker_domain::{MoodEntry, MoodScore};

pub struct UpdateMoodEntryService {
    entries: Arc<dyn MoodEntryRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateMoodEntryService {
    pub fn new(entries: Arc<dyn MoodEntryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { entries, clock }
    }
}

#[async_trait::async_trait]
impl UpdateMoodEntryHandler for UpdateMoodEntryService {
    async fn handle(&self, command: UpdateMoodEntry) -> Result<MoodEntry, ApplicationError> {
        let score = MoodScore::new(command.score)
            .map_err(|_| ApplicationError::InvalidInput("mood score must be between 1 and 5"))?;
        let note = normalize_note(command.note.as_deref())?;
        self.entries
            .update(
                command.user_id,
                command.entry_id,
                score,
                note,
                self.clock.now().await,
            )
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "update_mood_entry_service.tests.rs"]
mod tests;
