use super::create_user_command::CreateUserCommand;
use crate::ApplicationError;
use identity_domain::Role;

#[derive(Debug, Default, Clone, Copy)]
pub struct CreateUserCommandValidator;

impl CreateUserCommandValidator {
    pub fn validate(&self, command: &CreateUserCommand) -> Result<(), ApplicationError> {
        if command.actor.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_user_command_validator.tests.rs"]
mod tests;
