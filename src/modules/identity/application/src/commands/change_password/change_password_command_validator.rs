use super::change_password_command::ChangePasswordCommand;
use crate::ApplicationError;

/// Input rules of [`ChangePasswordCommand`]. Its fields are already valid by
/// construction (domain value types), so no rule rejects a command yet.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChangePasswordCommandValidator;

impl ChangePasswordCommandValidator {
    pub fn validate(&self, _command: &ChangePasswordCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "change_password_command_validator.tests.rs"]
mod tests;
