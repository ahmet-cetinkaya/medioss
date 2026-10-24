use std::sync::Arc;

use super::setup_admin::SetupAdmin;
use super::setup_admin_handler::SetupAdminHandler;
use crate::ports::{Clock, InitialAdminRegistration, PasswordHasher, UserRepository};
use crate::ApplicationError;
use identity_domain::User;

pub struct SetupAdminService {
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
}

impl SetupAdminService {
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
impl SetupAdminHandler for SetupAdminService {
    async fn handle(&self, command: SetupAdmin) -> Result<User, ApplicationError> {
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
#[path = "setup_admin_service.tests.rs"]
mod tests;
