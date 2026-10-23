use super::create_category::CreateCategory;
use crate::ApplicationError;
use soundscape_domain::SoundCategory;

#[async_trait::async_trait]
pub trait CreateCategoryHandler: Send + Sync {
    async fn handle(&self, command: CreateCategory) -> Result<SoundCategory, ApplicationError>;
}
