use super::authenticate::Authenticate;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait AuthenticateHandler: Send + Sync {
    async fn handle(&self, command: Authenticate) -> Result<User, ApplicationError>;
}
