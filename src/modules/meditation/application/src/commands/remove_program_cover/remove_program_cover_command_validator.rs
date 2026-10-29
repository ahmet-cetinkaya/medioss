use super::remove_program_cover_command::RemoveProgramCoverCommand;
use crate::ApplicationError;

/// Removing a cover has no input rule: existence is checked by the handler.
#[derive(Debug, Default, Clone, Copy)]
pub struct RemoveProgramCoverCommandValidator;

impl RemoveProgramCoverCommandValidator {
    pub fn validate(&self, _command: &RemoveProgramCoverCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "remove_program_cover_command_validator.tests.rs"]
mod tests;
