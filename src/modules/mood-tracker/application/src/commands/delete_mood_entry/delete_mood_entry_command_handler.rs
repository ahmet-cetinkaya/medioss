use std::sync::Arc;

use super::delete_mood_entry_command::DeleteMoodEntryCommand;
use super::delete_mood_entry_command_validator::DeleteMoodEntryCommandValidator;
use crate::ports::{CommandHandler, MoodEntryRepository};
use crate::ApplicationError;

pub struct DeleteMoodEntryCommandHandler {
    entries: Arc<dyn MoodEntryRepository>,
    validator: DeleteMoodEntryCommandValidator,
}

impl DeleteMoodEntryCommandHandler {
    pub fn new(entries: Arc<dyn MoodEntryRepository>) -> Self {
        Self {
            entries,
            validator: DeleteMoodEntryCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteMoodEntryCommand> for DeleteMoodEntryCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteMoodEntryCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
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

#[cfg(test)]
#[path = "delete_mood_entry_command_handler.tests.rs"]
mod tests;
