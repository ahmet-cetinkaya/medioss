use std::sync::Arc;

use super::create_category_command::CreateCategoryCommand;
use super::create_category_command_validator::CreateCategoryCommandValidator;
use crate::ports::CategoryRepository;
use crate::ports::CommandHandler;
use crate::ApplicationError;
use soundscape_domain::SoundCategory;

pub struct CreateCategoryCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    validator: CreateCategoryCommandValidator,
}

impl CreateCategoryCommandHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self {
            categories,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<CreateCategoryCommand> for CreateCategoryCommandHandler {
    type Output = SoundCategory;

    async fn handle(
        &self,
        command: CreateCategoryCommand,
    ) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&command)?;
        self.categories
            .create(&command.name)
            .await
            .map_err(ApplicationError::from_category_repository)
    }
}

#[cfg(test)]
#[path = "create_category_command_handler.tests.rs"]
mod tests;
