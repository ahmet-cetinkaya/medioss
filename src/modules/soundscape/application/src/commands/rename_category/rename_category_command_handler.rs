use std::sync::Arc;

use super::rename_category_command::RenameCategoryCommand;
use super::rename_category_command_validator::RenameCategoryCommandValidator;
use crate::ports::CategoryRepository;
use crate::ports::CommandHandler;
use crate::ApplicationError;
use soundscape_domain::SoundCategory;

pub struct RenameCategoryCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    validator: RenameCategoryCommandValidator,
}

impl RenameCategoryCommandHandler {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self {
            categories,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RenameCategoryCommand> for RenameCategoryCommandHandler {
    type Output = SoundCategory;

    async fn handle(
        &self,
        command: RenameCategoryCommand,
    ) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&command)?;
        self.categories
            .rename(command.id, &command.name)
            .await
            .map_err(ApplicationError::from_category_repository)?
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "rename_category_command_handler.tests.rs"]
mod tests;
