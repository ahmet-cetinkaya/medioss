use std::sync::Arc;

use super::create_mood_entry_command::CreateMoodEntryCommand;
use super::create_mood_entry_command_validator::CreateMoodEntryCommandValidator;
use crate::ports::{Clock, CommandHandler, MoodEntryRepository};
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

pub struct CreateMoodEntryCommandHandler {
    entries: Arc<dyn MoodEntryRepository>,
    clock: Arc<dyn Clock>,
    validator: CreateMoodEntryCommandValidator,
}

impl CreateMoodEntryCommandHandler {
    pub fn new(entries: Arc<dyn MoodEntryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            entries,
            clock,
            validator: CreateMoodEntryCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<CreateMoodEntryCommand> for CreateMoodEntryCommandHandler {
    type Output = MoodEntry;

    async fn handle(&self, command: CreateMoodEntryCommand) -> Result<MoodEntry, ApplicationError> {
        self.validator.validate(&command)?;
        let score = self.validator.score(&command)?;
        let note = self.validator.normalized_note(&command)?;
        self.entries
            .insert(command.user_id, score, note, self.clock.now().await)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "create_mood_entry_command_handler.tests.rs"]
mod tests;
