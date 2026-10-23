use std::sync::Arc;

use super::delete_category::DeleteCategory;
use super::delete_category_handler::DeleteCategoryHandler;
use crate::ports::{CategoryRepository, ProgramRepository};
use crate::ApplicationError;

pub struct DeleteCategoryService {
    categories: Arc<dyn CategoryRepository>,
    programs: Arc<dyn ProgramRepository>,
}

impl DeleteCategoryService {
    pub fn new(
        categories: Arc<dyn CategoryRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            categories,
            programs,
        }
    }
}

#[async_trait::async_trait]
impl DeleteCategoryHandler for DeleteCategoryService {
    async fn handle(&self, command: DeleteCategory) -> Result<(), ApplicationError> {
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
#[path = "delete_category_service.tests.rs"]
mod tests;
