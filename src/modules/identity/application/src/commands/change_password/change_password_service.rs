use std::sync::Arc;

use super::change_password::ChangePassword;
use super::change_password_handler::ChangePasswordHandler;
use crate::ports::{CredentialChange, PasswordHasher, UserRepository};
use crate::ApplicationError;
use identity_domain::User;
use kernel::UserId;

pub struct ChangePasswordService {
    repository: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl ChangePasswordService {
    pub fn new(repository: Arc<dyn UserRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { repository, hasher }
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
impl ChangePasswordHandler for ChangePasswordService {
    async fn handle(&self, command: ChangePassword) -> Result<User, ApplicationError> {
        self.change_password(command.user_id, &command.new_password)
            .await
    }
}

#[cfg(test)]
#[path = "change_password_service.tests.rs"]
mod tests;
