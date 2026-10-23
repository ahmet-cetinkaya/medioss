use super::create_category::CreateCategory;
use crate::ports::Category;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait CreateCategoryHandler: Send + Sync {
    async fn handle(&self, command: CreateCategory) -> Result<Category, ApplicationError>;
}
