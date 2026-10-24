use super::delete_note_command::DeleteNoteCommand;
use crate::ApplicationError;

/// Deleting a note has no input rule: ownership is enforced by the repository delete.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteNoteCommandValidator;

impl DeleteNoteCommandValidator {
    pub fn validate(&self, _command: &DeleteNoteCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_note_command_validator.tests.rs"]
mod tests;
