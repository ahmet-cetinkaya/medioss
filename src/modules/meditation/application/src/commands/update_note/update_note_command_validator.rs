use super::update_note_command::UpdateNoteCommand;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateNoteCommandValidator;

impl UpdateNoteCommandValidator {
    pub fn validate(&self, command: &UpdateNoteCommand) -> Result<(), ApplicationError> {
        if command.text.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "note text must not be empty",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "update_note_command_validator.tests.rs"]
mod tests;
