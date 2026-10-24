use std::sync::Arc;

use super::clear_avatar_command::ClearAvatarCommand;
use super::clear_avatar_command_validator::ClearAvatarCommandValidator;
use crate::ports::{AvatarStore, CommandHandler, UserRepository};
use crate::ApplicationError;

pub struct ClearAvatarCommandHandler {
    repository: Arc<dyn UserRepository>,
    store: Arc<dyn AvatarStore>,
    validator: ClearAvatarCommandValidator,
}

impl ClearAvatarCommandHandler {
    pub fn new(repository: Arc<dyn UserRepository>, store: Arc<dyn AvatarStore>) -> Self {
        Self {
            repository,
            store,
            validator: ClearAvatarCommandValidator::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<ClearAvatarCommand> for ClearAvatarCommandHandler {
    type Output = ();

    async fn handle(&self, command: ClearAvatarCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;

        let actor = &command.actor;
        let user = self
            .repository
            .find_by_id(actor.user_id())
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;

        if let Some(avatar) = user.avatar() {
            self.repository
                .set_profile(actor.user_id(), user.display_name().cloned(), None)
                .await
                .map_err(ApplicationError::from)?;
            let _ = self.store.remove(actor.user_id(), &avatar).await;
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "clear_avatar_command_handler.tests.rs"]
mod tests;
