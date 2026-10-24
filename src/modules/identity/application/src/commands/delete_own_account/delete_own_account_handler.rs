use super::delete_own_account::DeleteOwnAccount;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteOwnAccountHandler: Send + Sync {
    /// Runs every guard without mutating anything, so the composition root
    /// can refuse a request before purging other modules' data.
    async fn ensure_can_delete_own_account(
        &self,
        actor: &kernel::Actor,
    ) -> Result<(), ApplicationError>;

    async fn handle(&self, command: DeleteOwnAccount) -> Result<(), ApplicationError>;
}
