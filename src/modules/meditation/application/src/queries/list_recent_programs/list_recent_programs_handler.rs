use super::list_recent_programs::ListRecentPrograms;
use super::recent_program::RecentProgram;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ListRecentProgramsHandler: Send + Sync {
    async fn handle(
        &self,
        query: ListRecentPrograms,
    ) -> Result<Vec<RecentProgram>, ApplicationError>;
}
