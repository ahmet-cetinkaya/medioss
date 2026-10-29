use std::sync::Arc;

use super::remove_program_cover_command::RemoveProgramCoverCommand;
use super::remove_program_cover_command_validator::RemoveProgramCoverCommandValidator;
use crate::ports::{BlobStore, CommandHandler, ProgramRepository};
use crate::ApplicationError;

pub struct RemoveProgramCoverCommandHandler {
    programs: Arc<dyn ProgramRepository>,
    store: Arc<dyn BlobStore>,
    validator: RemoveProgramCoverCommandValidator,
}

impl RemoveProgramCoverCommandHandler {
    pub fn new(programs: Arc<dyn ProgramRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self {
            programs,
            store,
            validator: RemoveProgramCoverCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RemoveProgramCoverCommand> for RemoveProgramCoverCommandHandler {
    type Output = ();

    /// Idempotent: a program that already has no cover succeeds unchanged.
    async fn handle(&self, command: RemoveProgramCoverCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;

        let program = self
            .programs
            .find_by_id(command.program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        let Some(cover_key) = program.cover_storage_key else {
            return Ok(());
        };

        // The row is the source of truth: clear it first, then drop the bytes.
        // A failed blob delete only leaves an unreferenced file behind.
        self.programs
            .clear_cover_key(command.program_id)
            .await
            .map_err(ApplicationError::Repository)?;
        let _ = self.store.delete(&cover_key).await;
        Ok(())
    }
}

#[cfg(test)]
#[path = "remove_program_cover_command_handler.tests.rs"]
mod tests;
