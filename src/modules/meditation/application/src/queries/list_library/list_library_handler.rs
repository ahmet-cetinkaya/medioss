use super::list_library::ListLibrary;
use crate::ApplicationError;
use meditation_domain::Meditation;

/// A library meditation decorated with its home program, so the client can
/// render program links without a second request.
#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub meditation: Meditation,
    pub program_id: Option<i64>,
    pub program_has_cover: bool,
}

#[async_trait::async_trait]
pub trait ListLibraryHandler: Send + Sync {
    async fn handle(&self, query: ListLibrary) -> Result<Vec<LibraryEntry>, ApplicationError>;
}
