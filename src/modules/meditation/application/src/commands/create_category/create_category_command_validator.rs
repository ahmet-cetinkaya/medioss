use super::create_category_command::CreateCategoryCommand;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct CreateCategoryCommandValidator;

impl CreateCategoryCommandValidator {
    pub fn validate(&self, command: &CreateCategoryCommand) -> Result<(), ApplicationError> {
        if self.normalized_name(command).is_empty() {
            return Err(ApplicationError::InvalidInput("name must not be empty"));
        }
        Ok(())
    }

    /// The name as stored: surrounding whitespace never counts.
    pub fn normalized_name<'a>(&self, command: &'a CreateCategoryCommand) -> &'a str {
        command.name.trim()
    }
}

#[cfg(test)]
#[path = "create_category_command_validator.tests.rs"]
mod tests;
