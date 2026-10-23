use super::list_categories::ListCategories;
use crate::ports::Category;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ListCategoriesHandler: Send + Sync {
    async fn handle(&self, query: ListCategories) -> Result<Vec<Category>, ApplicationError>;
}
