use super::delete_user_as_admin::DeleteUserAsAdmin;
use crate::ApplicationError;
use kernel::UserId;

#[async_trait::async_trait]
pub trait DeleteUserAsAdminHandler: Send + Sync {
    /// Runs every guard without mutating anything, so the composition root
    /// can refuse a request before purging other modules' data.
    async fn ensure_admin_can_delete_user(
        &self,
        actor: &kernel::Actor,
        target: UserId,
    ) -> Result<(), ApplicationError>;

    async fn handle(&self, command: DeleteUserAsAdmin) -> Result<(), ApplicationError>;
}
