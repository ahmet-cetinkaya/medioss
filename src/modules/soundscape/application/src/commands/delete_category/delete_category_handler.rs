use super::delete_category::DeleteCategory;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteCategoryHandler: Send + Sync {
    async fn handle(&self, command: DeleteCategory) -> Result<(), ApplicationError>;
}
