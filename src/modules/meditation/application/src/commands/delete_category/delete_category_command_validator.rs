use super::delete_category_command::DeleteCategoryCommand;
use crate::ApplicationError;

/// The command carries only identifiers; existence is checked against the
/// repositories in the handler, so there is no input rule to enforce here.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeleteCategoryCommandValidator;

impl DeleteCategoryCommandValidator {
    pub fn validate(&self, _command: &DeleteCategoryCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_category_command_validator.tests.rs"]
mod tests;
