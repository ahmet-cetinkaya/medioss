use std::sync::Arc;

use super::list_program_members_query::ListProgramMembersQuery;
use super::list_program_members_query_validator::ListProgramMembersQueryValidator;
use crate::ports::Membership;
use crate::ports::MembershipRepository;
use crate::ports::ProgramRepository;
use crate::ports::QueryHandler;
use crate::ApplicationError;

pub struct ListProgramMembersQueryHandler {
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
    validator: ListProgramMembersQueryValidator,
}

impl ListProgramMembersQueryHandler {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        memberships: Arc<dyn MembershipRepository>,
    ) -> Self {
        Self {
            programs,
            memberships,
            validator: ListProgramMembersQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListProgramMembersQuery> for ListProgramMembersQueryHandler {
    type Output = Vec<Membership>;

    async fn handle(
        &self,
        query: ListProgramMembersQuery,
    ) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
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
#[path = "list_program_members_query_handler.tests.rs"]
mod tests;
