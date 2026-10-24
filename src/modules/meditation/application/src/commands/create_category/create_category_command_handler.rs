use std::sync::Arc;

use super::create_category_command::CreateCategoryCommand;
use super::create_category_command_validator::CreateCategoryCommandValidator;
use crate::ports::Category;
use crate::ports::{CategoryRepository, Clock, CommandHandler};
use crate::ApplicationError;

pub struct CreateCategoryCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    clock: Arc<dyn Clock>,
    validator: CreateCategoryCommandValidator,
}

impl CreateCategoryCommandHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            categories,
            clock,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<CreateCategoryCommand> for CreateCategoryCommandHandler {
    type Output = Category;

    async fn handle(&self, command: CreateCategoryCommand) -> Result<Category, ApplicationError> {
        self.validator.validate(&command)?;
        let name = self.validator.normalized_name(&command);
        if self
            .categories
            .find_by_name(name)
            .await
            .map_err(ApplicationError::Repository)?
            .is_some()
        {
            return Err(ApplicationError::DuplicateCategory);
        }
        self.categories
            .insert(name.to_owned(), self.clock.now().await)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "create_category_command_handler.tests.rs"]
mod tests;
