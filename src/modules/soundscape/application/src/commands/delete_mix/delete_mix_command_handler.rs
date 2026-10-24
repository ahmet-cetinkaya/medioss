use std::sync::Arc;

use super::delete_mix_command::DeleteMixCommand;
use super::delete_mix_command_validator::DeleteMixCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::MixRepository;
use crate::ApplicationError;

pub struct DeleteMixCommandHandler {
    mixes: Arc<dyn MixRepository>,
    validator: DeleteMixCommandValidator,
}

impl DeleteMixCommandHandler {
    pub fn new(mixes: Arc<dyn MixRepository>) -> Self {
        Self {
            mixes,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<DeleteMixCommand> for DeleteMixCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteMixCommand) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&command)?;
        if self
            .mixes
            .delete_for(command.id, command.user_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "delete_mix_command_handler.tests.rs"]
mod tests;
