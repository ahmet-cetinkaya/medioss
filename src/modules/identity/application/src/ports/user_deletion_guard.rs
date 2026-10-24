use crate::ApplicationError;

/// Runs every guard for an admin-initiated deletion without mutating
/// anything, so a refused request never destroys the target's module data.
#[async_trait::async_trait]
pub trait UserDeletionGuard: Send + Sync {
    async fn ensure_admin_can_delete_user(
        &self,
        actor: &kernel::Actor,
        target: kernel::UserId,
    ) -> Result<(), ApplicationError>;
}
