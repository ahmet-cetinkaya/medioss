use super::delete_own_account_command::DeleteOwnAccountCommand;
use crate::ApplicationError;

/// Self-deletion has no input rule: the actor is always the account owner.
/// The last-active-admin guard needs the repository, so the handler owns it.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteOwnAccountCommandValidator;

impl DeleteOwnAccountCommandValidator {
    pub fn validate(&self, _command: &DeleteOwnAccountCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_own_account_command_validator.tests.rs"]
mod tests;
