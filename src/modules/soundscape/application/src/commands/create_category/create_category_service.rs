use std::sync::Arc;

use super::create_category::CreateCategory;
use super::create_category_handler::CreateCategoryHandler;
use crate::ports::CategoryRepository;
use crate::ApplicationError;
use soundscape_domain::{valid_name, SoundCategory};

pub struct CreateCategoryService {
    categories: Arc<dyn CategoryRepository>,
}

impl CreateCategoryService {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self { categories }
    }
}

#[async_trait::async_trait]
impl CreateCategoryHandler for CreateCategoryService {
    async fn handle(&self, command: CreateCategory) -> Result<SoundCategory, ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("category name"))?;
        self.categories
            .create(&command.name)
            .await
            .map_err(ApplicationError::from_category_repository)
    }
}

#[cfg(test)]
#[path = "create_category_service.tests.rs"]
mod tests;
