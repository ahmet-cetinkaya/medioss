use std::sync::Arc;

use super::list_categories::ListCategories;
use super::list_categories_handler::ListCategoriesHandler;
use crate::ports::CategoryRepository;
use crate::ApplicationError;
use soundscape_domain::SoundCategory;

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
    async fn handle(&self, _query: ListCategories) -> Result<Vec<SoundCategory>, ApplicationError> {
        self.categories
            .list()
            .await
            .map_err(ApplicationError::Repository)
    }
}
