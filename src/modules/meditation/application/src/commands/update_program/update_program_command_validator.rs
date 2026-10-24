use super::update_program_command::UpdateProgramCommand;
use crate::services::validate_name;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateProgramCommandValidator;

impl UpdateProgramCommandValidator {
    pub fn validate(&self, command: &UpdateProgramCommand) -> Result<(), ApplicationError> {
        validate_name(&command.title)
    }
}

#[cfg(test)]
#[path = "update_program_command_validator.tests.rs"]
mod tests;
