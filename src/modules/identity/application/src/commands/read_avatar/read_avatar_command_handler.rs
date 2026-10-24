use std::sync::Arc;

use super::read_avatar_command::ReadAvatarCommand;
use super::read_avatar_command_validator::ReadAvatarCommandValidator;
use crate::ports::{AvatarStore, CommandHandler, UserRepository};
use crate::ApplicationError;
use identity_domain::AvatarRef;

pub struct ReadAvatarCommandHandler {
    repository: Arc<dyn UserRepository>,
    store: Arc<dyn AvatarStore>,
    validator: ReadAvatarCommandValidator,
}

impl ReadAvatarCommandHandler {
    pub fn new(repository: Arc<dyn UserRepository>, store: Arc<dyn AvatarStore>) -> Self {
        Self {
            repository,
            store,
            validator: ReadAvatarCommandValidator::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<ReadAvatarCommand> for ReadAvatarCommandHandler {
    type Output = (Vec<u8>, AvatarRef);

    async fn handle(
        &self,
        command: ReadAvatarCommand,
    ) -> Result<(Vec<u8>, AvatarRef), ApplicationError> {
        self.validator.validate(&command)?;

        let user = self
            .repository
            .find_by_id(command.requested)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        let avatar = user.avatar().ok_or(ApplicationError::UserNotFound)?;

        let bytes = self
            .store
            .read(command.requested, &avatar)
            .await
            .map_err(|_| ApplicationError::UserNotFound)?;

        Ok((bytes, avatar))
    }
}

#[cfg(test)]
#[path = "read_avatar_command_handler.tests.rs"]
mod tests;
