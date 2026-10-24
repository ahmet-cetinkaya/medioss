use super::authenticate_command::AuthenticateCommand;
use crate::models::normalize_login_identifier;
use crate::ApplicationError;

/// Input rules of [`AuthenticateCommand`]. Its fields are already valid by
/// construction, so no rule rejects a command yet; it owns the identifier's canonical form.
#[derive(Debug, Default, Clone, Copy)]
pub struct AuthenticateCommandValidator;

impl AuthenticateCommandValidator {
    pub fn validate(&self, _command: &AuthenticateCommand) -> Result<(), ApplicationError> {
        Ok(())
    }

    /// The identifier throttling and lookup key on.
    pub fn normalized_identifier(&self, command: &AuthenticateCommand) -> String {
        normalize_login_identifier(&command.identifier)
    }
}

#[cfg(test)]
#[path = "authenticate_command_validator.tests.rs"]
mod tests;
