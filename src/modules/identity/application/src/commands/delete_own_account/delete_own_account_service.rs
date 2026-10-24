use std::sync::Arc;

use super::delete_own_account::DeleteOwnAccount;
use super::delete_own_account_handler::DeleteOwnAccountHandler;
use crate::commands::delete_user_as_admin::last_active_admin_guard::ensure_not_last_active_admin;
use crate::ports::UserRepository;
use crate::ApplicationError;

pub struct DeleteOwnAccountService {
    repository: Arc<dyn UserRepository>,
}

impl DeleteOwnAccountService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    /// Runs every guard for a self-deletion without mutating anything. The
    /// composition root calls this before purging module data, so a refused
    /// request (last active admin) never destroys the caller's data.
    pub async fn ensure_can_delete_own_account(
        &self,
        actor: &kernel::Actor,
    ) -> Result<(), ApplicationError> {
        let user = self
            .repository
            .find_by_id(actor.user_id())
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        ensure_not_last_active_admin(self.repository.as_ref(), &user).await
    }

    async fn delete_own_account(&self, actor: &kernel::Actor) -> Result<(), ApplicationError> {
        self.ensure_can_delete_own_account(actor).await?;
        let user = self
            .repository
            .find_by_id(actor.user_id())
            .await
            .map_err(ApplicationError::from)?
            .ok_or(ApplicationError::UserNotFound)?;
        self.repository
            .delete_login_attempts(&user.email().as_str().trim().to_lowercase())
            .await
            .map_err(ApplicationError::from)?;
        let deleted = self
            .repository
            .delete_user(actor.user_id())
            .await
            .map_err(ApplicationError::from)?;
        if !deleted {
            return Err(ApplicationError::UserNotFound);
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl DeleteOwnAccountHandler for DeleteOwnAccountService {
    async fn ensure_can_delete_own_account(
        &self,
        actor: &kernel::Actor,
    ) -> Result<(), ApplicationError> {
        DeleteOwnAccountService::ensure_can_delete_own_account(self, actor).await
    }

    async fn handle(&self, command: DeleteOwnAccount) -> Result<(), ApplicationError> {
        self.delete_own_account(&command.actor).await
    }
}

#[cfg(test)]
#[path = "delete_own_account_service.tests.rs"]
mod tests;
