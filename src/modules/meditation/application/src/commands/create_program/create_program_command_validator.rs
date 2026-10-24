use super::create_program_command::CreateProgramCommand;
use super::create_program_command::MAX_COVER_BYTES;
use crate::services::validate_name;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct CreateProgramCommandValidator;

impl CreateProgramCommandValidator {
    pub fn validate(&self, command: &CreateProgramCommand) -> Result<(), ApplicationError> {
        validate_name(&command.title)?;
        let cover_too_large = command
            .cover_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > MAX_COVER_BYTES);
        if cover_too_large {
            return Err(ApplicationError::PayloadTooLarge);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_program_command_validator.tests.rs"]
mod tests;
