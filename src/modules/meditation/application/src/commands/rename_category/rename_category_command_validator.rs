use super::rename_category_command::RenameCategoryCommand;
use crate::ApplicationError;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenameCategoryCommandValidator;

impl RenameCategoryCommandValidator {
    pub fn validate(&self, command: &RenameCategoryCommand) -> Result<(), ApplicationError> {
        if self.normalized_name(command).is_empty() {
            return Err(ApplicationError::InvalidInput("name must not be empty"));
        }
        Ok(())
    }

    /// The name as stored: surrounding whitespace never counts.
    pub fn normalized_name<'a>(&self, command: &'a RenameCategoryCommand) -> &'a str {
        command.name.trim()
    }
}

#[cfg(test)]
#[path = "rename_category_command_validator.tests.rs"]
mod tests;
