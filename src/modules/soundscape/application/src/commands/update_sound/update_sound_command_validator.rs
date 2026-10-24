use super::update_sound_command::UpdateSoundCommand;
use crate::services::parse_icon_key;
use crate::ApplicationError;
use soundscape_domain::valid_name;

/// Name and bundled-glyph key rules; category existence is a repository check.
#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateSoundCommandValidator;

impl UpdateSoundCommandValidator {
    pub fn validate(&self, command: &UpdateSoundCommand) -> Result<(), ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
        parse_icon_key(command.icon_key.as_deref())?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "update_sound_command_validator.tests.rs"]
mod tests;
