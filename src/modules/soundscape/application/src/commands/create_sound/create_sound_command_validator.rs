use super::create_sound_command::CreateSoundCommand;
use super::create_sound_command::MAX_UPLOAD_BYTES;
use crate::services::parse_icon_key;
use crate::ApplicationError;
use soundscape_domain::valid_name;

/// Name, bundled-glyph key and upload-size rules; category existence and media sniffing need ports and stay in the handler.
#[derive(Debug, Default, Clone, Copy)]
pub struct CreateSoundCommandValidator;

impl CreateSoundCommandValidator {
    pub fn validate(&self, command: &CreateSoundCommand) -> Result<(), ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
        parse_icon_key(command.icon_key.as_deref())?;
        if command.bytes.len() > MAX_UPLOAD_BYTES {
            return Err(ApplicationError::PayloadTooLarge);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_sound_command_validator.tests.rs"]
mod tests;
