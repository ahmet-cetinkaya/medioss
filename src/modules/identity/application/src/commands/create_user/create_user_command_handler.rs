use std::sync::Arc;

use super::create_user_command::CreateUserCommand;
use super::create_user_command_validator::CreateUserCommandValidator;
use crate::ports::{Clock, CommandHandler, MemberRegistration, PasswordHasher, UserRepository};
use crate::ApplicationError;
use identity_domain::User;

pub struct CreateUserCommandHandler {
    validator: CreateUserCommandValidator,
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
}

impl CreateUserCommandHandler {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            validator: CreateUserCommandValidator,
            repository,
            hasher,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserCommandHandler {
    type Output = User;

    async fn handle(&self, command: CreateUserCommand) -> Result<User, ApplicationError> {
        self.validator.validate(&command)?;

        let password_hash = self.hasher.hash(&command.temporary_password).await?;
        let registration = MemberRegistration::new(command.email, password_hash, self.clock.now());

        self.repository
            .insert_member(registration)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "create_user_command_handler.tests.rs"]
mod tests;
