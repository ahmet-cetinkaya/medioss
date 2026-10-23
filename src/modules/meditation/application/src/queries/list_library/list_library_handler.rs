use super::list_library::ListLibrary;
use crate::ApplicationError;
use meditation_domain::Meditation;

#[async_trait::async_trait]
pub trait ListLibraryHandler: Send + Sync {
    async fn handle(&self, query: ListLibrary) -> Result<Vec<Meditation>, ApplicationError>;
}
