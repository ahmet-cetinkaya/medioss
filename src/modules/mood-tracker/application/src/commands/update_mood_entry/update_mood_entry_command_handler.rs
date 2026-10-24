use std::sync::Arc;

use super::update_mood_entry_command::UpdateMoodEntryCommand;
use super::update_mood_entry_command_validator::UpdateMoodEntryCommandValidator;
use crate::ports::{Clock, CommandHandler, MoodEntryRepository};
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

pub struct UpdateMoodEntryCommandHandler {
    entries: Arc<dyn MoodEntryRepository>,
    clock: Arc<dyn Clock>,
    validator: UpdateMoodEntryCommandValidator,
}

impl UpdateMoodEntryCommandHandler {
    pub fn new(entries: Arc<dyn MoodEntryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            entries,
            clock,
            validator: UpdateMoodEntryCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UpdateMoodEntryCommand> for UpdateMoodEntryCommandHandler {
    type Output = MoodEntry;

    async fn handle(&self, command: UpdateMoodEntryCommand) -> Result<MoodEntry, ApplicationError> {
        self.validator.validate(&command)?;
        let score = self.validator.score(&command)?;
        let note = self.validator.normalized_note(&command)?;
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
#[path = "update_mood_entry_command_handler.tests.rs"]
mod tests;
