use super::list_program_members::ListProgramMembers;
use crate::ports::Membership;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ListProgramMembersHandler: Send + Sync {
    async fn handle(&self, query: ListProgramMembers) -> Result<Vec<Membership>, ApplicationError>;
}
