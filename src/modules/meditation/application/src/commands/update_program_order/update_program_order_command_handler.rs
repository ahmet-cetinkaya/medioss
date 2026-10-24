use std::sync::Arc;

use super::update_program_order_command::UpdateProgramOrderCommand;
use crate::ports::CommandHandler;
use crate::ports::{MembershipRepository, ProgramRepository};
use crate::ApplicationError;

pub struct UpdateProgramOrderCommandHandler {
    programs: Arc<dyn ProgramRepository>,
    memberships: Arc<dyn MembershipRepository>,
}

impl UpdateProgramOrderCommandHandler {
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
impl CommandHandler<UpdateProgramOrderCommand> for UpdateProgramOrderCommandHandler {
    type Output = ();

    async fn handle(&self, command: UpdateProgramOrderCommand) -> Result<(), ApplicationError> {
        // `meditation_ids` must be exactly the program's current meditations,
        // each once; anything else is rejected.
        let UpdateProgramOrderCommand {
            program_id,
            meditation_ids,
        } = command;
        self.programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
        let mut current: Vec<i64> = self
            .memberships
            .list_by_program(program_id)
            .await
            .map_err(ApplicationError::Repository)?
            .into_iter()
            .map(|member| member.meditation_id)
            .collect();
        let mut requested = meditation_ids.clone();
        current.sort_unstable();
        requested.sort_unstable();
        if current != requested {
            return Err(ApplicationError::InvalidProgramOrder);
        }
        self.memberships
            .set_order(program_id, &meditation_ids)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "update_program_order_command_handler.tests.rs"]
mod tests;
