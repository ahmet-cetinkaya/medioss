use std::sync::Arc;

use super::delete_mood_entry::DeleteMoodEntry;
use super::delete_mood_entry_handler::DeleteMoodEntryHandler;
use crate::ports::MoodEntryRepository;
use crate::ApplicationError;

pub struct DeleteMoodEntryService {
    entries: Arc<dyn MoodEntryRepository>,
}

impl DeleteMoodEntryService {
    pub fn new(entries: Arc<dyn MoodEntryRepository>) -> Self {
        Self { entries }
    }
}

#[async_trait::async_trait]
impl DeleteMoodEntryHandler for DeleteMoodEntryService {
    async fn handle(&self, command: DeleteMoodEntry) -> Result<(), ApplicationError> {
        if self
            .entries
            .delete(command.user_id, command.entry_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}
