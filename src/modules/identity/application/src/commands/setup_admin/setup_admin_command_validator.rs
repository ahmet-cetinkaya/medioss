use super::setup_admin_command::SetupAdminCommand;
use crate::ApplicationError;

/// Input rules of [`SetupAdminCommand`]. Its fields are already valid by
/// construction (domain value types), so no rule rejects a command yet.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetupAdminCommandValidator;

impl SetupAdminCommandValidator {
    pub fn validate(&self, _command: &SetupAdminCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "setup_admin_command_validator.tests.rs"]
mod tests;
