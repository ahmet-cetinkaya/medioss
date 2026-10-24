use super::delete_mood_entry_command::DeleteMoodEntryCommand;
use crate::ApplicationError;

/// Deleting a mood entry has no input rule: ownership and existence are
/// decided by the repository.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteMoodEntryCommandValidator;

impl DeleteMoodEntryCommandValidator {
    pub fn validate(&self, _command: &DeleteMoodEntryCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_mood_entry_command_validator.tests.rs"]
mod tests;
