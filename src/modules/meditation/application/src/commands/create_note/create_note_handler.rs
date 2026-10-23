use super::create_note::CreateNote;
use crate::ports::Note;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait CreateNoteHandler: Send + Sync {
    async fn handle(&self, command: CreateNote) -> Result<Note, ApplicationError>;
}
