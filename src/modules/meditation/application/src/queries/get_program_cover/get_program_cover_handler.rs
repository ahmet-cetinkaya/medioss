use super::get_program_cover::GetProgramCover;
use crate::ApplicationError;

/// The cover bytes plus the sniffed content type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramCover {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

#[async_trait::async_trait]
pub trait GetProgramCoverHandler: Send + Sync {
    async fn handle(
        &self,
        query: GetProgramCover,
    ) -> Result<Option<ProgramCover>, ApplicationError>;
}
