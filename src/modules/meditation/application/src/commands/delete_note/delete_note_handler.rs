use super::delete_note::DeleteNote;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteNoteHandler: Send + Sync {
    async fn handle(&self, command: DeleteNote) -> Result<(), ApplicationError>;
}
