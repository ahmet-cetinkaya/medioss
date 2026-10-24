use super::clear_sound_icon_command::ClearSoundIconCommand;
use crate::ApplicationError;

/// Clearing an icon has no input rule: existence is checked against the repository.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClearSoundIconCommandValidator;

impl ClearSoundIconCommandValidator {
    pub fn validate(&self, _command: &ClearSoundIconCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "clear_sound_icon_command_validator.tests.rs"]
mod tests;
