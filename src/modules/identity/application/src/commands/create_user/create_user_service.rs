use std::sync::Arc;

use super::create_user::CreateUser;
use super::create_user_handler::CreateUserHandler;
use crate::ports::{Clock, MemberRegistration, PasswordHasher, UserRepository};
use crate::ApplicationError;
use identity_domain::{Role, User};

pub struct CreateUserService {
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
}

impl CreateUserService {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            repository,
            hasher,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CreateUserHandler for CreateUserService {
    async fn handle(&self, command: CreateUser) -> Result<User, ApplicationError> {
        if command.actor.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }

        let password_hash = self.hasher.hash(&command.temporary_password).await?;
        let registration = MemberRegistration::new(command.email, password_hash, self.clock.now());

        self.repository
            .insert_member(registration)
            .await
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
#[path = "create_user_service.tests.rs"]
mod tests;
