use super::rename_meditation_command::RenameMeditationCommand;
use crate::services::validate_name;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenameMeditationCommandValidator;

impl RenameMeditationCommandValidator {
    pub fn validate(&self, command: &RenameMeditationCommand) -> Result<(), ApplicationError> {
        validate_name(&command.name)
    }

    /// The name as stored: surrounding whitespace never counts.
    pub fn normalized_name<'a>(&self, command: &'a RenameMeditationCommand) -> &'a str {
        command.name.trim()
    }
}

#[cfg(test)]
#[path = "rename_meditation_command_validator.tests.rs"]
mod tests;
