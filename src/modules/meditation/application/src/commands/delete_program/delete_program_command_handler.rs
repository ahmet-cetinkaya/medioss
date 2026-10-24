use std::sync::Arc;

use super::delete_program_command::DeleteProgramCommand;
use super::delete_program_command::UNFILED;
use super::delete_program_command_validator::DeleteProgramCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{BlobStore, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

pub struct DeleteProgramCommandHandler {
    validator: DeleteProgramCommandValidator,
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
    meditations: Arc<dyn MeditationRepository>,
    store: Arc<dyn BlobStore>,
}

impl DeleteProgramCommandHandler {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        memberships: Arc<dyn MembershipRepository>,
        meditations: Arc<dyn MeditationRepository>,
        store: Arc<dyn BlobStore>,
    ) -> Self {
        Self {
            validator: DeleteProgramCommandValidator,
            programs,
            memberships,
            meditations,
            store,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteProgramCommand> for DeleteProgramCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteProgramCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        // Every meditation whose file actually lives in the program's folder is
        // relocated to `_unfiled/` first (decided by `storage_key` prefix, not
        // by the membership snapshot, so a concurrent upload or a missing
        // membership row can never leave an orphaned or destroyed blob); only
        // once every file is safely out of the folder are the memberships and
        // the program row removed and the folder deleted.
        //
        // Idempotent and safe to retry: a meditation whose key no longer
        // starts with the program's folder prefix is treated as already
        // relocated, so a retry after a partial failure only finishes the
        // remaining work (spec FR-028).
        let program_id = command.program_id;
        let Some(program) = self
            .programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
        else {
            return Err(ApplicationError::NotFound);
        };
        let prefix = format!("{}/", program.folder);
        for meditation in self
            .meditations
            .list_by_storage_prefix(&prefix)
            .await
            .map_err(ApplicationError::Repository)?
        {
            let new_key = self
                .store
                .relocate(meditation.storage_key(), UNFILED, meditation.name(), "opus")
                .await
                .map_err(ApplicationError::Storage)?;
            self.meditations
                .set_storage_key(meditation.id(), &new_key)
                .await
                .map_err(ApplicationError::Repository)?;
        }
        for membership in self
            .memberships
            .list_by_program(program_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            self.memberships
                .remove(program_id, membership.meditation_id)
                .await
                .map_err(ApplicationError::Repository)?;
        }
        if self
            .programs
            .delete(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .is_none()
        {
            // Deleted concurrently between the lookup above and here: the
            // relocation and membership cleanup already ran, so this is not an
            // error — report success rather than a confusing NotFound.
            return Ok(());
        }
        self.store
            .delete_folder(&program.folder)
            .await
            .map_err(ApplicationError::Storage)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "delete_program_command_handler.tests.rs"]
mod tests;
