use super::replace_program_cover_command::ReplaceProgramCoverCommand;
use crate::ApplicationError;

/// The cover bytes arrive already validated and normalized, so there is no input rule.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReplaceProgramCoverCommandValidator;

impl ReplaceProgramCoverCommandValidator {
    pub fn validate(&self, _command: &ReplaceProgramCoverCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "replace_program_cover_command_validator.tests.rs"]
mod tests;
