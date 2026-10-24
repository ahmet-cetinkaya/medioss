use super::read_avatar_command::ReadAvatarCommand;
use crate::ApplicationError;
use kernel::Role;

#[derive(Debug, Default, Clone, Copy)]
pub struct ReadAvatarCommandValidator;

impl ReadAvatarCommandValidator {
    pub fn validate(&self, command: &ReadAvatarCommand) -> Result<(), ApplicationError> {
        if command.reader.user_id() != command.requested && command.reader.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "read_avatar_command_validator.tests.rs"]
mod tests;
