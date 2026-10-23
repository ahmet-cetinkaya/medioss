use std::sync::Arc;

use super::list_notes::ListNotes;
use super::list_notes_handler::ListNotesHandler;
use crate::ports::Note;
use crate::ports::{MeditationRepository, NoteRepository};
use crate::services::load_meditation;
use crate::ApplicationError;

pub struct ListNotesService {
    meditations: Arc<dyn MeditationRepository>,
    notes: Arc<dyn NoteRepository>,
}

impl ListNotesService {
    pub fn new(meditations: Arc<dyn MeditationRepository>, notes: Arc<dyn NoteRepository>) -> Self {
        Self { meditations, notes }
    }
}

#[async_trait::async_trait]
impl ListNotesHandler for ListNotesService {
    async fn handle(&self, query: ListNotes) -> Result<Vec<Note>, ApplicationError> {
        // Newest first (spec FR-035).
        let ListNotes {
            user_id,
            meditation_id,
        } = query;
        load_meditation(self.meditations.as_ref(), meditation_id).await?;
        self.notes
            .list_for(user_id, meditation_id)
            .await
            .map_err(ApplicationError::Repository)
    }
}
