use std::sync::Arc;

use super::delete_category_command::DeleteCategoryCommand;
use super::delete_category_command_validator::DeleteCategoryCommandValidator;
use crate::ports::{CategoryRepository, CommandHandler, ProgramRepository};
use crate::ApplicationError;

pub struct DeleteCategoryCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    programs: Arc<dyn ProgramRepository>,
    validator: DeleteCategoryCommandValidator,
}

impl DeleteCategoryCommandHandler {
    pub fn new(
        categories: Arc<dyn CategoryRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            categories,
            programs,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteCategoryCommand> for DeleteCategoryCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteCategoryCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        let category_id = command.category_id;
        // Sweep references before removing the row: if the sweep failed after
        // the delete already committed, programs would keep a `category_id`
        // pointing at nothing (no FK enforcement). Sweeping first only risks a
        // harmless no-op sweep if the category never existed.
        self.programs
            .clear_category(category_id)
            .await
            .map_err(ApplicationError::Repository)?;
        let deleted = self
            .categories
            .delete(category_id)
            .await
            .map_err(ApplicationError::Repository)?;
        if !deleted {
            return Err(ApplicationError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_category_command_handler.tests.rs"]
mod tests;
