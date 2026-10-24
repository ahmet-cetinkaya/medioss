use super::replace_avatar_command::ReplaceAvatarCommand;
use crate::ApplicationError;

/// Image content rules (format, size) belong to the avatar store's
/// `normalize`, which needs to decode the bytes, so nothing is checked here.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReplaceAvatarCommandValidator;

impl ReplaceAvatarCommandValidator {
    pub fn validate(&self, _command: &ReplaceAvatarCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "replace_avatar_command_validator.tests.rs"]
mod tests;
