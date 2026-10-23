use std::sync::Arc;

use super::delete_meditation::DeleteMeditation;
use super::delete_meditation_handler::DeleteMeditationHandler;
use crate::ports::{BlobStore, MeditationRepository, MembershipRepository, NoteRepository};
use crate::ApplicationError;

pub struct DeleteMeditationService {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    notes: Arc<dyn NoteRepository>,
    store: Arc<dyn BlobStore>,
}

impl DeleteMeditationService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        notes: Arc<dyn NoteRepository>,
        store: Arc<dyn BlobStore>,
    ) -> Self {
        Self {
            meditations,
            memberships,
            notes,
            store,
        }
    }
}

#[async_trait::async_trait]
impl DeleteMeditationHandler for DeleteMeditationService {
    async fn handle(&self, command: DeleteMeditation) -> Result<(), ApplicationError> {
        // Row removal is the source of truth: stored bytes are removed after
        // the row is gone, and a store failure surfaces without resurrecting
        // the row (spec FR-014).
        let meditation_id = command.meditation_id;
        let Some(meditation) = self
            .meditations
            .find_by_id(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
        else {
            return Err(ApplicationError::NotFound);
        };
        self.memberships
            .remove_for_meditation(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
        self.notes
            .delete_for_meditation(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
        self.meditations
            .delete(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
        self.store
            .delete(meditation.storage_key())
            .await
            .map_err(ApplicationError::Storage)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_meditation_service.tests.rs"]
mod tests;
