use super::delete_category_command::DeleteCategoryCommand;
use crate::ApplicationError;

/// Deleting by id has no input rule beyond what the repository reports.
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
