use super::set_program_category_command::SetProgramCategoryCommand;
use crate::ApplicationError;

/// The command carries only identifiers; existence is checked against the
/// repositories in the handler, so there is no input rule to enforce here.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetProgramCategoryCommandValidator;

impl SetProgramCategoryCommandValidator {
    pub fn validate(&self, _command: &SetProgramCategoryCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "set_program_category_command_validator.tests.rs"]
mod tests;
