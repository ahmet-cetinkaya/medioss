use super::create_program::CreateProgram;
use crate::ports::Program;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait CreateProgramHandler: Send + Sync {
    async fn handle(&self, command: CreateProgram) -> Result<Program, ApplicationError>;
}
