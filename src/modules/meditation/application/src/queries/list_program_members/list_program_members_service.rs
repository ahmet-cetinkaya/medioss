use std::sync::Arc;

use super::list_program_members::ListProgramMembers;
use super::list_program_members_handler::ListProgramMembersHandler;
use crate::ports::Membership;
use crate::ports::MembershipRepository;
use crate::ports::ProgramRepository;
use crate::ApplicationError;

pub struct ListProgramMembersService {
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
}

impl ListProgramMembersService {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        memberships: Arc<dyn MembershipRepository>,
    ) -> Self {
        Self {
            programs,
            memberships,
        }
    }
}

#[async_trait::async_trait]
impl ListProgramMembersHandler for ListProgramMembersService {
    async fn handle(&self, query: ListProgramMembers) -> Result<Vec<Membership>, ApplicationError> {
        // A missing program is 404, not an empty list (matches the route's
        // previous existence gate).
        if self
            .programs
            .find_by_id(query.program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .is_none()
        {
            return Err(ApplicationError::NotFound);
        }
        // Ordered by `position` (spec FR-027).
        self.memberships
            .list_by_program(query.program_id)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_program_members_service.tests.rs"]
mod tests;
