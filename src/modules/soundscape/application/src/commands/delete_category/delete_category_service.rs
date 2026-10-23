use std::sync::Arc;

use super::delete_category::DeleteCategory;
use super::delete_category_handler::DeleteCategoryHandler;
use crate::ports::{CategoryDelete, CategoryRepository};
use crate::ApplicationError;

pub struct DeleteCategoryService {
    categories: Arc<dyn CategoryRepository>,
}

impl DeleteCategoryService {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self { categories }
    }
}

#[async_trait::async_trait]
impl DeleteCategoryHandler for DeleteCategoryService {
    async fn handle(&self, command: DeleteCategory) -> Result<(), ApplicationError> {
        match self
            .categories
            .delete_empty(command.id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            CategoryDelete::Deleted => Ok(()),
            CategoryDelete::NotFound => Err(ApplicationError::NotFound),
            CategoryDelete::InUse => Err(ApplicationError::CategoryInUse),
        }
    }
}
