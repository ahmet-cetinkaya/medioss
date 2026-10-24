use std::sync::Arc;

use super::remove_member::RemoveMember;
use super::remove_member_handler::RemoveMemberHandler;
use crate::ports::{MembershipRepository, ProgramRepository};
use crate::ApplicationError;

pub struct RemoveMemberService {
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
}

impl RemoveMemberService {
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
impl RemoveMemberHandler for RemoveMemberService {
    async fn handle(&self, command: RemoveMember) -> Result<(), ApplicationError> {
        // Unknown pairs succeed idempotently (spec FR-027).
        let RemoveMember {
            program_id,
            meditation_id,
        } = command;
        self.programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        self.memberships
            .remove(program_id, meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "remove_member_service.tests.rs"]
mod tests;
