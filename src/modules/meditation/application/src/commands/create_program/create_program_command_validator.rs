use super::create_program_command::CreateProgramCommand;
use crate::services::validate_name;
use crate::ApplicationError;

/// The cover size cap is injected at the composition root (spec FR-026).
#[derive(Debug, Clone, Copy)]
pub struct CreateProgramCommandValidator {
    max_cover_bytes: usize,
}

impl CreateProgramCommandValidator {
    pub fn new(max_cover_bytes: usize) -> Self {
        Self { max_cover_bytes }
    }

    pub fn validate(&self, command: &CreateProgramCommand) -> Result<(), ApplicationError> {
        validate_name(&command.title)?;
        let cover_too_large = command
            .cover_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > self.max_cover_bytes);
        if cover_too_large {
            return Err(ApplicationError::PayloadTooLarge);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_program_command_validator.tests.rs"]
mod tests;
