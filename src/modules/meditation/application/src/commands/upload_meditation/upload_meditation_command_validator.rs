use super::upload_meditation_command::UploadMeditationCommand;
use crate::services::{validate_description, validate_name};
use crate::ApplicationError;

/// Name, description and payload size rules that need nothing but the command.
/// The size cap applies to the original bytes (spec FR-001/002).
#[derive(Debug, Clone, Copy)]
pub struct UploadMeditationCommandValidator {
    max_upload_bytes: usize,
}

impl UploadMeditationCommandValidator {
    pub fn new(max_upload_bytes: usize) -> Self {
        Self { max_upload_bytes }
    }

    pub fn validate(&self, command: &UploadMeditationCommand) -> Result<(), ApplicationError> {
        validate_name(&command.name)?;
        self.normalized_description(command)?;
        if command.bytes.len() > self.max_upload_bytes {
            return Err(ApplicationError::PayloadTooLarge);
        }
        Ok(())
    }

    pub fn normalized_name(&self, command: &UploadMeditationCommand) -> String {
        command.name.trim().to_owned()
    }

    pub fn normalized_description(
        &self,
        command: &UploadMeditationCommand,
    ) -> Result<Option<String>, ApplicationError> {
        validate_description(command.description.as_deref())
    }
}

#[cfg(test)]
#[path = "upload_meditation_command_validator.tests.rs"]
mod tests;
