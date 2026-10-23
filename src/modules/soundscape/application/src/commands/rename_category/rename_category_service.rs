use std::sync::Arc;

use super::rename_category::RenameCategory;
use super::rename_category_handler::RenameCategoryHandler;
use crate::ports::CategoryRepository;
use crate::ApplicationError;
use soundscape_domain::{valid_name, SoundCategory};

pub struct RenameCategoryService {
    categories: Arc<dyn CategoryRepository>,
}

impl RenameCategoryService {
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self { categories }
    }
}

#[async_trait::async_trait]
impl RenameCategoryHandler for RenameCategoryService {
    async fn handle(&self, command: RenameCategory) -> Result<SoundCategory, ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("category name"))?;
        self.categories
            .rename(command.id, &command.name)
            .await
            .map_err(ApplicationError::from_category_repository)?
            .ok_or(ApplicationError::NotFound)
    }
}
