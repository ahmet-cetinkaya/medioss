use std::sync::Arc;

use super::create_note::CreateNote;
use super::create_note_handler::CreateNoteHandler;
use crate::ports::Note;
use crate::ports::{Clock, MeditationRepository, NoteRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct CreateNoteService {
    meditations: Arc<dyn MeditationRepository>,
    notes: Arc<dyn NoteRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateNoteService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        notes: Arc<dyn NoteRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            meditations,
            notes,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CreateNoteHandler for CreateNoteService {
    async fn handle(&self, command: CreateNote) -> Result<Note, ApplicationError> {
        let CreateNote {
            user_id,
            meditation_id,
            text,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        let text = text.trim();
        if text.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "note text must not be empty",
            ));
        }
        let now = self.clock.now().await;
        self.notes
            .insert(user_id, meditation_id, text.to_owned(), now)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "create_note_service.tests.rs"]
mod tests;
