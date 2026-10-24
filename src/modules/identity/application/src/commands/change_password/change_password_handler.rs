use super::change_password::ChangePassword;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait ChangePasswordHandler: Send + Sync {
    async fn handle(&self, command: ChangePassword) -> Result<User, ApplicationError>;
}
