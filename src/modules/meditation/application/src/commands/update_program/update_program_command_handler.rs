use std::sync::Arc;

use super::update_program_command::UpdateProgramCommand;
use super::update_program_command_validator::UpdateProgramCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::ProgramRepository;
use crate::ApplicationError;

pub struct UpdateProgramCommandHandler {
    validator: UpdateProgramCommandValidator,
    programs: Arc<dyn ProgramRepository>,
}

impl UpdateProgramCommandHandler {
    pub fn new(programs: Arc<dyn ProgramRepository>) -> Self {
        Self {
            validator: UpdateProgramCommandValidator,
            programs,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UpdateProgramCommand> for UpdateProgramCommandHandler {
    type Output = ();

    async fn handle(&self, command: UpdateProgramCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        // Updates title and description (spec FR-025).
        let UpdateProgramCommand {
            program_id,
            title,
            description,
        } = command;
        let updated = self
            .programs
            .update(
                program_id,
                title.trim().to_owned(),
                description
                    .as_deref()
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_owned),
            )
            .await
            .map_err(ApplicationError::Repository)?;
        if updated {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "update_program_command_handler.tests.rs"]
mod tests;
