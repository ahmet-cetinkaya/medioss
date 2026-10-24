use std::sync::Arc;

use super::setup_admin_command::SetupAdminCommand;
use super::setup_admin_command_validator::SetupAdminCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{Clock, InitialAdminRegistration, PasswordHasher, UserRepository};
use crate::ApplicationError;
use identity_domain::User;

pub struct SetupAdminCommandHandler {
    validator: SetupAdminCommandValidator,
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
}

impl SetupAdminCommandHandler {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            validator: Default::default(),
            repository,
            hasher,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<SetupAdminCommand> for SetupAdminCommandHandler {
    type Output = User;

    async fn handle(&self, command: SetupAdminCommand) -> Result<User, ApplicationError> {
        self.validator.validate(&command)?;
        let password_hash = self.hasher.hash(&command.password).await?;
        let registration =
            InitialAdminRegistration::new(command.email, password_hash, self.clock.now());

        self.repository
            .insert_initial_admin_if_uninitialized(registration)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "setup_admin_command_handler.tests.rs"]
mod tests;
