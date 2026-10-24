use std::sync::Arc;

use super::replace_program_cover_command::ReplaceProgramCoverCommand;
use super::replace_program_cover_command_validator::ReplaceProgramCoverCommandValidator;
use crate::ports::{BlobStore, CommandHandler, ProgramRepository};
use crate::ApplicationError;

pub struct ReplaceProgramCoverCommandHandler {
    programs: Arc<dyn ProgramRepository>,
    store: Arc<dyn BlobStore>,
    validator: ReplaceProgramCoverCommandValidator,
}

impl ReplaceProgramCoverCommandHandler {
    pub fn new(programs: Arc<dyn ProgramRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self {
            programs,
            store,
            validator: ReplaceProgramCoverCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<ReplaceProgramCoverCommand> for ReplaceProgramCoverCommandHandler {
    type Output = ();

    async fn handle(&self, command: ReplaceProgramCoverCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;

        let program = self
            .programs
            .find_by_id(command.program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;

        let blob = self
            .store
            .put(&program.folder, "cover", "webp", &command.bytes)
            .await
            .map_err(ApplicationError::Storage)?;
        self.programs
            .set_cover_key(command.program_id, &blob.storage_key)
            .await
            .map_err(ApplicationError::Repository)?;
        if let Some(old_key) = program.cover_storage_key {
            if old_key != blob.storage_key {
                let _ = self.store.delete(&old_key).await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "replace_program_cover_command_handler.tests.rs"]
mod tests;
