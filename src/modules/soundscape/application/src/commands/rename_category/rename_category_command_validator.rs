use super::rename_category_command::RenameCategoryCommand;
use crate::ApplicationError;
use soundscape_domain::valid_name;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenameCategoryCommandValidator;

impl RenameCategoryCommandValidator {
    pub fn validate(&self, command: &RenameCategoryCommand) -> Result<(), ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("category name"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "rename_category_command_validator.tests.rs"]
mod tests;
