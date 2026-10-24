use std::sync::Arc;

use super::get_program_cover_query::GetProgramCoverQuery;
use super::get_program_cover_query_validator::GetProgramCoverQueryValidator;
use super::program_cover::ProgramCover;
use crate::ports::{BlobStore, ProgramRepository, QueryHandler};
use crate::ApplicationError;

pub struct GetProgramCoverQueryHandler {
    programs: Arc<dyn ProgramRepository>,
    store: Arc<dyn BlobStore>,
    validator: GetProgramCoverQueryValidator,
}

impl GetProgramCoverQueryHandler {
    pub fn new(programs: Arc<dyn ProgramRepository>, store: Arc<dyn BlobStore>) -> Self {
        Self {
            programs,
            store,
            validator: Default::default(),
        }
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
impl QueryHandler<GetProgramCoverQuery> for GetProgramCoverQueryHandler {
    type Output = Option<ProgramCover>;

    async fn handle(
        &self,
        query: GetProgramCoverQuery,
    ) -> Result<Option<ProgramCover>, ApplicationError> {
        self.validator.validate(&query)?;
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

#[cfg(test)]
#[path = "get_program_cover_query_handler.tests.rs"]
mod tests;
