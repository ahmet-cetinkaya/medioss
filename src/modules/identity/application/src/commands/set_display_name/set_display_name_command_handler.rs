use std::sync::Arc;

use super::set_display_name_command::SetDisplayNameCommand;
use super::set_display_name_command_validator::SetDisplayNameCommandValidator;
use crate::ports::{CommandHandler, UserRepository};
use crate::ApplicationError;
use identity_domain::User;
use kernel::UserId;

pub struct SetDisplayNameCommandHandler {
    repository: Arc<dyn UserRepository>,
    validator: SetDisplayNameCommandValidator,
}

impl SetDisplayNameCommandHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            repository,
            validator: SetDisplayNameCommandValidator,
        }
    }

    async fn set_display_name(
        &self,
        user_id: UserId,
        display_name: Option<identity_domain::DisplayName>,
    ) -> Result<User, ApplicationError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;

        self.repository
            .set_profile(user_id, display_name, user.avatar())
            .await
            .map_err(ApplicationError::from)
    }
}

#[async_trait::async_trait]
impl CommandHandler<SetDisplayNameCommand> for SetDisplayNameCommandHandler {
    type Output = User;

    async fn handle(&self, command: SetDisplayNameCommand) -> Result<User, ApplicationError> {
        self.validator.validate(&command)?;

        self.set_display_name(command.user_id, command.display_name)
            .await
    }
}

#[cfg(test)]
#[path = "set_display_name_command_handler.tests.rs"]
mod tests;
