use std::sync::Arc;

use super::set_program_category::SetProgramCategory;
use super::set_program_category_handler::SetProgramCategoryHandler;
use crate::ports::{CategoryRepository, ProgramRepository};
use crate::ApplicationError;

pub struct SetProgramCategoryService {
    programs: Arc<dyn ProgramRepository>,
    categories: Arc<dyn CategoryRepository>,
}

impl SetProgramCategoryService {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        categories: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            programs,
            categories,
        }
    }
}

#[async_trait::async_trait]
impl SetProgramCategoryHandler for SetProgramCategoryService {
    async fn handle(&self, command: SetProgramCategory) -> Result<(), ApplicationError> {
        // Assigns a category, or clears it with `None` (spec FR-031).
        let SetProgramCategory {
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
#[path = "set_program_category_service.tests.rs"]
mod tests;
