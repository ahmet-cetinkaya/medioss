use super::setup_admin::SetupAdmin;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait SetupAdminHandler: Send + Sync {
    async fn handle(&self, command: SetupAdmin) -> Result<User, ApplicationError>;
}
