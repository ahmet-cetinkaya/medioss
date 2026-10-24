use std::sync::Arc;

use super::list_notes_query::ListNotesQuery;
use super::list_notes_query_validator::ListNotesQueryValidator;
use crate::ports::Note;
use crate::ports::QueryHandler;
use crate::ports::{MeditationRepository, NoteRepository};
use crate::services::load_meditation;
use crate::ApplicationError;

pub struct ListNotesQueryHandler {
    meditations: Arc<dyn MeditationRepository>,
    notes: Arc<dyn NoteRepository>,
    validator: ListNotesQueryValidator,
}

impl ListNotesQueryHandler {
    pub fn new(meditations: Arc<dyn MeditationRepository>, notes: Arc<dyn NoteRepository>) -> Self {
        Self {
            meditations,
            notes,
            validator: ListNotesQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListNotesQuery> for ListNotesQueryHandler {
    type Output = Vec<Note>;

    async fn handle(&self, query: ListNotesQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        // Newest first (spec FR-035).
        let ListNotesQuery {
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

#[cfg(test)]
#[path = "list_notes_query_handler.tests.rs"]
mod tests;
