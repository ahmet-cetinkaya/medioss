use std::sync::Arc;

use super::delete_user_as_admin::DeleteUserAsAdmin;
use super::delete_user_as_admin_handler::DeleteUserAsAdminHandler;
use super::last_active_admin_guard::ensure_not_last_active_admin;
use crate::ports::UserRepository;
use crate::ApplicationError;
use kernel::Role;

pub struct DeleteUserAsAdminService {
    repository: Arc<dyn UserRepository>,
}

impl DeleteUserAsAdminService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    /// Runs every guard for an admin-initiated deletion without mutating
    /// anything. The composition root calls this before purging module
    /// data, so a refused request never destroys the target's data.
    pub async fn ensure_admin_can_delete_user(
        &self,
        actor: &kernel::Actor,
        target: kernel::UserId,
    ) -> Result<(), ApplicationError> {
        if actor.role() != Role::Admin {
            return Err(ApplicationError::Forbidden);
        }
        let user = self
            .repository
            .find_by_id(target)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        ensure_not_last_active_admin(self.repository.as_ref(), &user).await
    }

    async fn delete_user_as_admin(
        &self,
        actor: &kernel::Actor,
        target: kernel::UserId,
    ) -> Result<(), ApplicationError> {
        self.ensure_admin_can_delete_user(actor, target).await?;
        let user = self
            .repository
            .find_by_id(target)
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        self.repository
            .delete_login_attempts(&user.email().as_str().trim().to_lowercase())
            .await
            .map_err(ApplicationError::from)?;
        let deleted = self
            .repository
            .delete_user(target)
            .await
            .map_err(ApplicationError::from)?;
        if !deleted {
            return Err(ApplicationError::UserNotFound);
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl DeleteUserAsAdminHandler for DeleteUserAsAdminService {
    async fn ensure_admin_can_delete_user(
        &self,
        actor: &kernel::Actor,
        target: kernel::UserId,
    ) -> Result<(), ApplicationError> {
        DeleteUserAsAdminService::ensure_admin_can_delete_user(self, actor, target).await
    }

    async fn handle(&self, command: DeleteUserAsAdmin) -> Result<(), ApplicationError> {
        self.delete_user_as_admin(&command.actor, command.target)
            .await
    }
}

#[cfg(test)]
#[path = "delete_user_as_admin_service.tests.rs"]
mod tests;
