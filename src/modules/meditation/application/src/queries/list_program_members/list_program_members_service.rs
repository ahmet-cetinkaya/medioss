use std::sync::Arc;

use super::list_program_members::ListProgramMembers;
use super::list_program_members_handler::ListProgramMembersHandler;
use crate::ports::Membership;
use crate::ports::MembershipRepository;
use crate::ApplicationError;

pub struct ListProgramMembersService {
    memberships: Arc<dyn MembershipRepository>,
}

impl ListProgramMembersService {
    pub fn new(memberships: Arc<dyn MembershipRepository>) -> Self {
        Self { memberships }
    }
}

#[async_trait::async_trait]
impl ListProgramMembersHandler for ListProgramMembersService {
    async fn handle(&self, query: ListProgramMembers) -> Result<Vec<Membership>, ApplicationError> {
        // Ordered by `position` (spec FR-027).
        self.memberships
            .list_by_program(query.program_id)
            .await
            .map_err(ApplicationError::Repository)
    }
}
