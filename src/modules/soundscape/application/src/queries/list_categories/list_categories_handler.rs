use super::list_categories::ListCategories;
use crate::ApplicationError;
use soundscape_domain::SoundCategory;

#[async_trait::async_trait]
pub trait ListCategoriesHandler: Send + Sync {
    async fn handle(&self, query: ListCategories) -> Result<Vec<SoundCategory>, ApplicationError>;
}
