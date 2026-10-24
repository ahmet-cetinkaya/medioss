use super::delete_user_as_admin_command::DeleteUserAsAdminCommand;
use crate::ApplicationError;
use kernel::Role;

#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteUserAsAdminCommandValidator;

impl DeleteUserAsAdminCommandValidator {
    pub fn validate(&self, command: &DeleteUserAsAdminCommand) -> Result<(), ApplicationError> {
        if command.actor.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_user_as_admin_command_validator.tests.rs"]
mod tests;
