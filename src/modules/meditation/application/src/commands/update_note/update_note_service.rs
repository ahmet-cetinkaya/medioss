use std::sync::Arc;

use super::update_note::UpdateNote;
use super::update_note_handler::UpdateNoteHandler;
use crate::ports::{Clock, NoteRepository};
use crate::ApplicationError;

pub struct UpdateNoteService {
    notes: Arc<dyn NoteRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateNoteService {
    pub fn new(notes: Arc<dyn NoteRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { notes, clock }
    }
}

#[async_trait::async_trait]
impl UpdateNoteHandler for UpdateNoteService {
    async fn handle(&self, command: UpdateNote) -> Result<(), ApplicationError> {
        // Another author's, unknown, or mismatched-meditation id is NotFound
        // (spec FR-036/037).
        let UpdateNote {
            note_id,
            meditation_id,
            user_id,
            text,
        } = command;
        let text = text.trim();
        if text.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "note text must not be empty",
            ));
        }
        let updated = self
            .notes
            .update(
                note_id,
                meditation_id,
                user_id,
                text.to_owned(),
                self.clock.now().await,
            )
            .await
            .map_err(ApplicationError::Repository)?;
        if updated {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}
