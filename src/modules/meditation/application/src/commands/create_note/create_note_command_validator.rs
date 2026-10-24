use super::create_note_command::CreateNoteCommand;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct CreateNoteCommandValidator;

impl CreateNoteCommandValidator {
    pub fn validate(&self, command: &CreateNoteCommand) -> Result<(), ApplicationError> {
        if command.text.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "note text must not be empty",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_note_command_validator.tests.rs"]
mod tests;
