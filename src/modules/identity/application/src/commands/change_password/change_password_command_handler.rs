use std::sync::Arc;

use super::change_password_command::ChangePasswordCommand;
use super::change_password_command_validator::ChangePasswordCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{CredentialChange, PasswordHasher, UserRepository};
use crate::ApplicationError;
use identity_domain::User;
use kernel::UserId;

pub struct ChangePasswordCommandHandler {
    validator: ChangePasswordCommandValidator,
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl ChangePasswordCommandHandler {
    pub fn new(repository: Arc<dyn UserRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self {
            validator: Default::default(),
            repository,
            hasher,
        }
    }

    async fn change_password(
        &self,
        user_id: UserId,
        new_password: &identity_domain::RawPassword,
    ) -> Result<User, ApplicationError> {
        let password_hash = self.hasher.hash(new_password).await?;
        let credential_change = CredentialChange::permanent(user_id, password_hash);

        self.repository
            .apply_credential_change(credential_change)
            .await
            .map_err(ApplicationError::from)
    }
}

#[async_trait::async_trait]
impl CommandHandler<ChangePasswordCommand> for ChangePasswordCommandHandler {
    type Output = User;

    async fn handle(&self, command: ChangePasswordCommand) -> Result<User, ApplicationError> {
        self.validator.validate(&command)?;
        self.change_password(command.user_id, &command.new_password)
            .await
    }
}

#[cfg(test)]
#[path = "change_password_command_handler.tests.rs"]
mod tests;
