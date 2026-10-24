use std::sync::Arc;

use super::list_categories::ListCategories;
use super::list_categories_handler::ListCategoriesHandler;
use crate::ports::Category;
use crate::ports::CategoryRepository;
use crate::ApplicationError;

pub struct ListCategoriesService {
    categories: Arc<dyn CategoryRepository>,
}

impl ListCategoriesService {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self { categories }
    }
}

#[async_trait::async_trait]
impl ListCategoriesHandler for ListCategoriesService {
    async fn handle(&self, _query: ListCategories) -> Result<Vec<Category>, ApplicationError> {
        // Lists every category (spec FR-030 views).
        self.categories
            .list_all()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_categories_service.tests.rs"]
mod tests;
