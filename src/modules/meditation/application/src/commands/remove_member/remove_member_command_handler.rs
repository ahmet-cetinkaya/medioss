use std::sync::Arc;

use super::remove_member_command::RemoveMemberCommand;
use super::remove_member_command_validator::RemoveMemberCommandValidator;
use crate::ports::{CommandHandler, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

pub struct RemoveMemberCommandHandler {
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
    validator: RemoveMemberCommandValidator,
}

impl RemoveMemberCommandHandler {
    pub fn new(
        programs: Arc<dyn ProgramRepository>,
        memberships: Arc<dyn MembershipRepository>,
    ) -> Self {
        Self {
            programs,
            memberships,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RemoveMemberCommand> for RemoveMemberCommandHandler {
    type Output = ();

    async fn handle(&self, command: RemoveMemberCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        // Unknown pairs succeed idempotently (spec FR-027).
        let RemoveMemberCommand {
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
#[path = "remove_member_command_handler.tests.rs"]
mod tests;
