use std::sync::Arc;

use super::rename_category_command::RenameCategoryCommand;
use super::rename_category_command_validator::RenameCategoryCommandValidator;
use crate::ports::RenameCategoryError;
use crate::ports::{CategoryRepository, Clock, CommandHandler};
use crate::ApplicationError;

pub struct RenameCategoryCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    clock: Arc<dyn Clock>,
    validator: RenameCategoryCommandValidator,
}

impl RenameCategoryCommandHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            categories,
            clock,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RenameCategoryCommand> for RenameCategoryCommandHandler {
    type Output = ();

    async fn handle(&self, command: RenameCategoryCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        let name = self.validator.normalized_name(&command);
        let category_id = command.category_id;
        // A duplicate check against every other category, so renaming to the
        // same name (a no-op) stays allowed.
        let existing = self
            .categories
            .find_by_name(name)
            .await
            .map_err(ApplicationError::Repository)?;
        if existing.is_some_and(|other| other.id != category_id) {
            return Err(ApplicationError::DuplicateCategory);
        }
        let renamed = self
            .categories
            .rename(category_id, name.to_owned(), self.clock.now().await)
            .await
            .map_err(|error| match error {
                RenameCategoryError::Duplicate => ApplicationError::DuplicateCategory,
                RenameCategoryError::Repository(error) => ApplicationError::Repository(error),
            })?;
        if !renamed {
            return Err(ApplicationError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "rename_category_command_handler.tests.rs"]
mod tests;
