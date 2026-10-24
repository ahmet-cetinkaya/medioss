use super::delete_sound_command::DeleteSoundCommand;
use crate::ApplicationError;

/// Deleting a sound has no input rule: existence is checked against the repository.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteSoundCommandValidator;

impl DeleteSoundCommandValidator {
    pub fn validate(&self, _command: &DeleteSoundCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_sound_command_validator.tests.rs"]
mod tests;
