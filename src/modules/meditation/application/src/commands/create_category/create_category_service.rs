use std::sync::Arc;

use super::create_category::CreateCategory;
use super::create_category_handler::CreateCategoryHandler;
use crate::ports::Category;
use crate::ports::{CategoryRepository, Clock};
use crate::ApplicationError;

pub struct CreateCategoryService {
    categories: Arc<dyn CategoryRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateCategoryService {
    pub fn new(categories: Arc<dyn CategoryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { categories, clock }
    }
}

#[async_trait::async_trait]
impl CreateCategoryHandler for CreateCategoryService {
    async fn handle(&self, command: CreateCategory) -> Result<Category, ApplicationError> {
        let name = command.name.trim();
        if name.is_empty() {
            return Err(ApplicationError::InvalidInput("name must not be empty"));
        }
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
#[path = "create_category_service.tests.rs"]
mod tests;
