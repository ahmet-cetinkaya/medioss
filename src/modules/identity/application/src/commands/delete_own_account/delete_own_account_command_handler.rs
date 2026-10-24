use std::sync::Arc;

use super::delete_own_account_command::DeleteOwnAccountCommand;
use super::delete_own_account_command_validator::DeleteOwnAccountCommandValidator;
use crate::commands::delete_user_as_admin::last_active_admin_guard::ensure_not_last_active_admin;
use crate::models::normalize_login_identifier;
use crate::ports::{CommandHandler, OwnAccountDeletionGuard, UserRepository};
use crate::ApplicationError;

pub struct DeleteOwnAccountCommandHandler {
    validator: DeleteOwnAccountCommandValidator,
    repository: Arc<dyn UserRepository>,
}

impl DeleteOwnAccountCommandHandler {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            validator: DeleteOwnAccountCommandValidator,
            repository,
        }
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
            .delete_login_attempts(&normalize_login_identifier(user.email().as_str()))
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
impl OwnAccountDeletionGuard for DeleteOwnAccountCommandHandler {
    async fn ensure_can_delete_own_account(
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
}

#[async_trait::async_trait]
impl CommandHandler<DeleteOwnAccountCommand> for DeleteOwnAccountCommandHandler {
    type Output = ();

    async fn handle(&self, command: DeleteOwnAccountCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        self.delete_own_account(&command.actor).await
    }
}

#[cfg(test)]
#[path = "delete_own_account_command_handler.tests.rs"]
mod tests;
