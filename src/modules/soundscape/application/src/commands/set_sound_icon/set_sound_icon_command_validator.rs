use super::set_sound_icon_command::SetSoundIconCommand;
use crate::ApplicationError;

/// Icon bytes are sniffed and size-checked by the image normalizer (a port), so there is no pure input rule.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetSoundIconCommandValidator;

impl SetSoundIconCommandValidator {
    pub fn validate(&self, _command: &SetSoundIconCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "set_sound_icon_command_validator.tests.rs"]
mod tests;
