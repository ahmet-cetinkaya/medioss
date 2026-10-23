use super::list_programs::ListPrograms;
use crate::ports::Program;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ListProgramsHandler: Send + Sync {
    async fn handle(&self, query: ListPrograms) -> Result<Vec<Program>, ApplicationError>;
}
