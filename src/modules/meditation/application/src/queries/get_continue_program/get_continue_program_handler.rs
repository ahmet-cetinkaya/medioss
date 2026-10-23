use super::continue_program::ContinueProgram;
use super::get_continue_program::GetContinueProgram;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetContinueProgramHandler: Send + Sync {
    async fn handle(
        &self,
        query: GetContinueProgram,
    ) -> Result<Option<ContinueProgram>, ApplicationError>;
}
