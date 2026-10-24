use std::sync::Arc;

use super::set_user_status_command::SetUserStatusCommand;
use super::set_user_status_command_validator::SetUserStatusCommandValidator;
use crate::ports::{CommandHandler, UserRepository};
use crate::ApplicationError;
use kernel::Role;

pub struct SetUserStatusCommandHandler {
    validator: SetUserStatusCommandValidator,
    repository: Arc<dyn UserRepository>,
}

impl SetUserStatusCommandHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            validator: SetUserStatusCommandValidator,
            repository,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<SetUserStatusCommand> for SetUserStatusCommandHandler {
    type Output = ();

    async fn handle(&self, command: SetUserStatusCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        let user = self
            .repository
            .find_by_id(command.target)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        if command.disabled && user.role() == Role::Admin {
            let active_admins = self
                .repository
                .count_active_admins()
                .await
                .map_err(ApplicationError::from)?;
            if active_admins <= 1 {
                return Err(ApplicationError::LastActiveAdmin);
            }
        }
        self.repository
            .set_disabled(command.target, command.disabled)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "set_user_status_command_handler.tests.rs"]
mod tests;
