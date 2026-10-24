use super::set_completed_command::SetCompletedCommand;
use crate::ApplicationError;

/// Setting the completed flag has no input rule: both values are valid.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetCompletedCommandValidator;

impl SetCompletedCommandValidator {
    pub fn validate(&self, _command: &SetCompletedCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "set_completed_command_validator.tests.rs"]
mod tests;
