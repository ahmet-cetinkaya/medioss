use super::delete_program::DeleteProgram;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteProgramHandler: Send + Sync {
    async fn handle(&self, command: DeleteProgram) -> Result<(), ApplicationError>;
}
