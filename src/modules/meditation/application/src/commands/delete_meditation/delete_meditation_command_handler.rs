use std::sync::Arc;

use super::delete_meditation_command::DeleteMeditationCommand;
use super::delete_meditation_command_validator::DeleteMeditationCommandValidator;
use crate::ports::{
    BlobStore, CommandHandler, FavoriteRepository, MeditationRepository, MembershipRepository,
    NoteRepository,
};
use crate::ApplicationError;

pub struct DeleteMeditationCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    notes: Arc<dyn NoteRepository>,
    favorites: Arc<dyn FavoriteRepository>,
    store: Arc<dyn BlobStore>,
    validator: DeleteMeditationCommandValidator,
}

impl DeleteMeditationCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        notes: Arc<dyn NoteRepository>,
        favorites: Arc<dyn FavoriteRepository>,
        store: Arc<dyn BlobStore>,
    ) -> Self {
        Self {
            meditations,
            memberships,
            notes,
            favorites,
            store,
            validator: DeleteMeditationCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteMeditationCommand> for DeleteMeditationCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteMeditationCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;

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
        self.favorites
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
#[path = "delete_meditation_command_handler.tests.rs"]
mod tests;
