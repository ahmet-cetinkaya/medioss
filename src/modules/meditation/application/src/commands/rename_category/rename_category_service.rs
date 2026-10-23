use std::sync::Arc;

use super::rename_category::RenameCategory;
use super::rename_category_handler::RenameCategoryHandler;
use crate::ports::RenameCategoryError;
use crate::ports::{CategoryRepository, Clock};
use crate::ApplicationError;

pub struct RenameCategoryService {
    categories: Arc<dyn CategoryRepository>,
    clock: Arc<dyn Clock>,
}

impl RenameCategoryService {
    pub fn new(categories: Arc<dyn CategoryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { categories, clock }
    }
}

#[async_trait::async_trait]
impl RenameCategoryHandler for RenameCategoryService {
    async fn handle(&self, command: RenameCategory) -> Result<(), ApplicationError> {
        let RenameCategory { category_id, name } = command;
        let name = name.trim();
        if name.is_empty() {
            return Err(ApplicationError::InvalidInput("name must not be empty"));
        }
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
#[path = "rename_category_service.tests.rs"]
mod tests;
