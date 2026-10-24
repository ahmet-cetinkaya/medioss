use super::delete_meditation_command::DeleteMeditationCommand;
use crate::ApplicationError;

/// Deleting a meditation has no input rule: existence is checked by the handler.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteMeditationCommandValidator;

impl DeleteMeditationCommandValidator {
    pub fn validate(&self, _command: &DeleteMeditationCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_meditation_command_validator.tests.rs"]
mod tests;
