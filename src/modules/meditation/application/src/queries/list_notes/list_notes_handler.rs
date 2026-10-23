use super::list_notes::ListNotes;
use crate::ports::Note;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ListNotesHandler: Send + Sync {
    async fn handle(&self, query: ListNotes) -> Result<Vec<Note>, ApplicationError>;
}
