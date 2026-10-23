use std::sync::Arc;

use super::create_program::CreateProgram;
use super::create_program::MAX_COVER_BYTES;
use super::create_program_handler::CreateProgramHandler;
use crate::ports::Program;
use crate::ports::{BlobStore, Clock, CoverValidator, ProgramRepository};
use crate::services::validate_name;
use crate::ApplicationError;

pub struct CreateProgramService {
    programs: Arc<dyn ProgramRepository>,
    cover_validator: Arc<dyn CoverValidator>,
    store: Arc<dyn BlobStore>,
    clock: Arc<dyn Clock>,
}

impl CreateProgramService {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        cover_validator: Arc<dyn CoverValidator>,
        store: Arc<dyn BlobStore>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            programs,
            cover_validator,
            store,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CreateProgramHandler for CreateProgramService {
    async fn handle(&self, command: CreateProgram) -> Result<Program, ApplicationError> {
        let CreateProgram {
            title,
            description,
            cover_bytes,
        } = command;
        validate_name(&title)?;
        let cover = match cover_bytes {
            Some(bytes) => {
                if bytes.len() > MAX_COVER_BYTES {
                    return Err(ApplicationError::PayloadTooLarge);
                }
                let normalized = self
                    .cover_validator
                    .normalize(bytes)
                    .await
                    .map_err(ApplicationError::from)?;
                Some((normalized, "webp"))
            }
            None => None,
        };
        let program = self
            .programs
            .insert(
                title.trim().to_owned(),
                description
                    .as_deref()
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_owned),
                self.clock.now().await,
            )
            .await
            .map_err(ApplicationError::Repository)?;
        if let Some((bytes, ext)) = cover {
            let blob = match self.store.put(&program.folder, "cover", ext, &bytes).await {
                Ok(blob) => blob,
                Err(error) => {
                    // The program row without a cover is partial state the
                    // caller never asked for (they submitted title+cover
                    // together) — undo it rather than leave a coverless program
                    // and a 503.
                    let _ = self.programs.delete(program.id).await;
                    return Err(ApplicationError::Storage(error));
                }
            };
            if let Err(error) = self
                .programs
                .set_cover_key(program.id, &blob.storage_key)
                .await
            {
                let _ = self.store.delete(&blob.storage_key).await;
                let _ = self.programs.delete(program.id).await;
                return Err(ApplicationError::Repository(error));
            }
        }
        self.programs
            .find_by_id(program.id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "create_program_service.tests.rs"]
mod tests;
