use super::authenticate_command::AuthenticateCommand;
use crate::ApplicationError;

/// Input rules of [`AuthenticateCommand`]. Its fields are already valid by
/// construction (domain value types), so no rule rejects a command yet.
#[derive(Debug, Default, Clone, Copy)]
pub struct AuthenticateCommandValidator;

impl AuthenticateCommandValidator {
    pub fn validate(&self, _command: &AuthenticateCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "authenticate_command_validator.tests.rs"]
mod tests;
