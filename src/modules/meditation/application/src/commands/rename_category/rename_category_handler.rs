use super::rename_category::RenameCategory;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait RenameCategoryHandler: Send + Sync {
    async fn handle(&self, command: RenameCategory) -> Result<(), ApplicationError>;
}
