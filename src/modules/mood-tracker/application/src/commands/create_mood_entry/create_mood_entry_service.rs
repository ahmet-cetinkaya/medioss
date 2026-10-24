use std::sync::Arc;

use super::create_mood_entry::CreateMoodEntry;
use super::create_mood_entry_handler::CreateMoodEntryHandler;
use super::normalize_note::normalize_note;
use crate::ports::{Clock, MoodEntryRepository};
use crate::ApplicationError;
use mood_tracker_domain::{MoodEntry, MoodScore};

pub struct CreateMoodEntryService {
    entries: Arc<dyn MoodEntryRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateMoodEntryService {
    pub fn new(entries: Arc<dyn MoodEntryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { entries, clock }
    }
}

#[async_trait::async_trait]
impl CreateMoodEntryHandler for CreateMoodEntryService {
    async fn handle(&self, command: CreateMoodEntry) -> Result<MoodEntry, ApplicationError> {
        let score = MoodScore::new(command.score).map_err(|_| {
            ApplicationError::InvalidInput("mood score must be between 1 and 5")
        })?;
        let note = normalize_note(command.note.as_deref())?;
        self.entries
            .insert(
                command.user_id,
                score,
                note,
                self.clock.now().await,
            )
            .await
            .map_err(ApplicationError::Repository)
    }
}
