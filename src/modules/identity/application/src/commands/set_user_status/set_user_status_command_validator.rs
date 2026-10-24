use super::set_user_status_command::SetUserStatusCommand;
use crate::ApplicationError;
use kernel::Role;

#[derive(Debug, Default, Clone, Copy)]
pub struct SetUserStatusCommandValidator;

impl SetUserStatusCommandValidator {
    pub fn validate(&self, command: &SetUserStatusCommand) -> Result<(), ApplicationError> {
        if command.actor.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }
        if command.actor.user_id() == command.target && command.disabled {
            return Err(ApplicationError::LastActiveAdmin);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "set_user_status_command_validator.tests.rs"]
mod tests;
