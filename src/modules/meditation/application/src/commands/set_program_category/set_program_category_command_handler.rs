use std::sync::Arc;

use super::set_program_category_command::SetProgramCategoryCommand;
use super::set_program_category_command_validator::SetProgramCategoryCommandValidator;
use crate::ports::{CategoryRepository, CommandHandler, ProgramRepository};
use crate::ApplicationError;

pub struct SetProgramCategoryCommandHandler {
    programs: Arc<dyn ProgramRepository>,
    categories: Arc<dyn CategoryRepository>,
    validator: SetProgramCategoryCommandValidator,
}

impl SetProgramCategoryCommandHandler {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        categories: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            programs,
            categories,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<SetProgramCategoryCommand> for SetProgramCategoryCommandHandler {
    type Output = ();

    async fn handle(&self, command: SetProgramCategoryCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        // Assigns a category, or clears it with `None` (spec FR-031).
        let SetProgramCategoryCommand {
            program_id,
            category_id,
        } = command;
        if let Some(category_id) = category_id {
            self.categories
                .find_by_id(category_id)
                .await
                .map_err(ApplicationError::Repository)?
                .ok_or(ApplicationError::NotFound)?;
        }
        let set = self
            .programs
            .set_category(program_id, category_id)
            .await
            .map_err(ApplicationError::Repository)?;
        if set {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "set_program_category_command_handler.tests.rs"]
mod tests;
