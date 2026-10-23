use super::update_program::UpdateProgram;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait UpdateProgramHandler: Send + Sync {
    async fn handle(&self, command: UpdateProgram) -> Result<(), ApplicationError>;
}
