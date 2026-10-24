use super::set_display_name_command::SetDisplayNameCommand;
use crate::ApplicationError;

/// The `DisplayName` value object already enforces format and length when it
/// is parsed, so the command carries no further input rule.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetDisplayNameCommandValidator;

impl SetDisplayNameCommandValidator {
    pub fn validate(&self, _command: &SetDisplayNameCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "set_display_name_command_validator.tests.rs"]
mod tests;
