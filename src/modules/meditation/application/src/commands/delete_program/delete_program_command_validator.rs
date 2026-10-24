use super::delete_program_command::DeleteProgramCommand;
use crate::ApplicationError;

/// Deleting a program has no input rule: existence is checked by the handler.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteProgramCommandValidator;

impl DeleteProgramCommandValidator {
    pub fn validate(&self, _command: &DeleteProgramCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_program_command_validator.tests.rs"]
mod tests;
