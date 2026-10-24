use super::clear_avatar_command::ClearAvatarCommand;
use crate::ApplicationError;

/// Clearing an avatar has no input rule: the actor always owns its own avatar.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClearAvatarCommandValidator;

impl ClearAvatarCommandValidator {
    pub fn validate(&self, _command: &ClearAvatarCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "clear_avatar_command_validator.tests.rs"]
mod tests;
