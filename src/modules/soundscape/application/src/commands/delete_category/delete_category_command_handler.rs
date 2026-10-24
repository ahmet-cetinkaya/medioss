use std::sync::Arc;

use super::delete_category_command::DeleteCategoryCommand;
use super::delete_category_command_validator::DeleteCategoryCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{CategoryDelete, CategoryRepository};
use crate::ApplicationError;

pub struct DeleteCategoryCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    validator: DeleteCategoryCommandValidator,
}

impl DeleteCategoryCommandHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self {
            categories,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteCategoryCommand> for DeleteCategoryCommandHandler {
    type Output = ();

    async fn handle(
        &self,
        command: DeleteCategoryCommand,
    ) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&command)?;
        match self
            .categories
            .delete_empty(command.id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            CategoryDelete::Deleted => Ok(()),
            CategoryDelete::NotFound => Err(ApplicationError::NotFound),
            CategoryDelete::InUse => Err(ApplicationError::CategoryInUse),
        }
    }
}

#[cfg(test)]
#[path = "delete_category_command_handler.tests.rs"]
mod tests;
