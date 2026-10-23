use super::update_note::UpdateNote;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait UpdateNoteHandler: Send + Sync {
    async fn handle(&self, command: UpdateNote) -> Result<(), ApplicationError>;
}
