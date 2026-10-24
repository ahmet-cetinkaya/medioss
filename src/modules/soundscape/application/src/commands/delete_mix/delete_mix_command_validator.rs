use super::delete_mix_command::DeleteMixCommand;
use crate::ApplicationError;

/// Deleting by id has no input rule beyond what the repository reports.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteMixCommandValidator;

impl DeleteMixCommandValidator {
    pub fn validate(&self, _command: &DeleteMixCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_mix_command_validator.tests.rs"]
mod tests;
