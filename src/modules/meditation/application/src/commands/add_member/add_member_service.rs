use std::sync::Arc;

use super::add_member::AddMember;
use super::add_member_handler::AddMemberHandler;
use crate::ports::{Clock, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct AddMemberService {
    meditations: Arc<dyn MeditationRepository>,
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
    clock: Arc<dyn Clock>,
}

impl AddMemberService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        programs: Arc<dyn ProgramRepository>,
        memberships: Arc<dyn MembershipRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            meditations,
            programs,
            memberships,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl AddMemberHandler for AddMemberService {
    async fn handle(&self, command: AddMember) -> Result<(), ApplicationError> {
        let AddMember {
            program_id,
            meditation_id,
        } = command;
        ensure_meditation_exists(self.meditations.as_ref(), meditation_id).await?;
        self.programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        self.memberships
            .add(program_id, meditation_id, self.clock.now().await)
            .await
            .map_err(ApplicationError::Repository)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "add_member_service.tests.rs"]
mod tests;
