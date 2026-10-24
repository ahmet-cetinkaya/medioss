use std::sync::Arc;

use super::get_program_cover::GetProgramCover;
use super::get_program_cover_handler::{GetProgramCoverHandler, ProgramCover};
use crate::ports::{BlobStore, ProgramRepository};
use crate::ApplicationError;

pub struct GetProgramCoverService {
    programs: Arc<dyn ProgramRepository>,
    store: Arc<dyn BlobStore>,
}

impl GetProgramCoverService {
    pub fn new(programs: Arc<dyn ProgramRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self { programs, store }
    }
}

fn sniff_content_type(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else {
        "image/webp"
    }
}

#[async_trait::async_trait]
impl GetProgramCoverHandler for GetProgramCoverService {
    async fn handle(
        &self,
        query: GetProgramCover,
    ) -> Result<Option<ProgramCover>, ApplicationError> {
        let program = self
            .programs
            .find_by_id(query.program_id)
            .await
            .map_err(ApplicationError::Repository)?;
        let Some(key) = program.and_then(|program| program.cover_storage_key) else {
            return Ok(None);
        };
        let bytes = self
            .store
            .read(&key)
            .await
            .map_err(ApplicationError::Storage)?;
        let content_type = sniff_content_type(&bytes);
        Ok(Some(ProgramCover {
            bytes,
            content_type,
        }))
    }
}
