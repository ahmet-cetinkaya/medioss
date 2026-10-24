use super::replace_program_cover::ReplaceProgramCover;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ReplaceProgramCoverHandler: Send + Sync {
    async fn handle(&self, command: ReplaceProgramCover) -> Result<(), ApplicationError>;
}
