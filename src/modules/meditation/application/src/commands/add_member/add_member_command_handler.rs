use std::sync::Arc;

use super::add_member_command::AddMemberCommand;
use super::add_member_command_validator::AddMemberCommandValidator;
use crate::ports::{
    Clock, CommandHandler, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

pub struct AddMemberCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
    clock: Arc<dyn Clock>,
    validator: AddMemberCommandValidator,
}

impl AddMemberCommandHandler {
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
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<AddMemberCommand> for AddMemberCommandHandler {
    type Output = ();

    async fn handle(&self, command: AddMemberCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        let AddMemberCommand {
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
#[path = "add_member_command_handler.tests.rs"]
mod tests;
