use crate::ApplicationError;

/// Runs every guard for a self-deletion without mutating anything, so the
/// composition root can refuse a request (last active admin) before purging
/// other modules' data.
#[async_trait::async_trait]
pub trait OwnAccountDeletionGuard: Send + Sync {
    async fn ensure_can_delete_own_account(
        &self,
        actor: &kernel::Actor,
    ) -> Result<(), ApplicationError>;
}
