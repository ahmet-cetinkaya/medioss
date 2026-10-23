use std::sync::Arc;

use super::delete_note::DeleteNote;
use super::delete_note_handler::DeleteNoteHandler;
use crate::ports::NoteRepository;
use crate::ApplicationError;

pub struct DeleteNoteService {
    notes: Arc<dyn NoteRepository>,
}

impl DeleteNoteService {
    pub fn new(notes: Arc<dyn NoteRepository>) -> Self {
        Self { notes }
    }
}

#[async_trait::async_trait]
impl DeleteNoteHandler for DeleteNoteService {
    async fn handle(&self, command: DeleteNote) -> Result<(), ApplicationError> {
        let DeleteNote {
            note_id,
            meditation_id,
            user_id,
        } = command;
        let deleted = self
            .notes
            .delete(note_id, meditation_id, user_id)
            .await
            .map_err(ApplicationError::Repository)?;
        if deleted {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}
