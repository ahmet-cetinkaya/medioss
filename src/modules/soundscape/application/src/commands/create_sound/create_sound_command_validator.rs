use super::create_sound_command::CreateSoundCommand;
use crate::services::parse_icon_key;
use crate::ApplicationError;
use soundscape_domain::valid_name;

/// Name, bundled-glyph key and upload-size rules; category existence and media sniffing need ports and stay in the handler.
#[derive(Debug, Clone, Copy)]
pub struct CreateSoundCommandValidator {
    max_upload_bytes: usize,
}

impl CreateSoundCommandValidator {
    pub fn new(max_upload_bytes: usize) -> Self {
        Self { max_upload_bytes }
    }

    pub fn validate(&self, command: &CreateSoundCommand) -> Result<(), ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
        parse_icon_key(command.icon_key.as_deref())?;
        if command.media.size_bytes > self.max_upload_bytes as u64 {
            return Err(ApplicationError::PayloadTooLarge);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_sound_command_validator.tests.rs"]
mod tests;
