use super::create_category_command::CreateCategoryCommand;
use crate::ApplicationError;
use soundscape_domain::valid_name;

#[derive(Debug, Default, Clone, Copy)]
pub struct CreateCategoryCommandValidator;

impl CreateCategoryCommandValidator {
    pub fn validate(&self, command: &CreateCategoryCommand) -> Result<(), ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("category name"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "create_category_command_validator.tests.rs"]
mod tests;
