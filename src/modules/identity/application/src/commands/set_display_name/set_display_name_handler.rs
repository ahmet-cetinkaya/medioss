use super::set_display_name::SetDisplayName;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait SetDisplayNameHandler: Send + Sync {
    async fn handle(&self, command: SetDisplayName) -> Result<User, ApplicationError>;
}
