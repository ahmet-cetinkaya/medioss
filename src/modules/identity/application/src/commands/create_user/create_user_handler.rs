use super::create_user::CreateUser;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait CreateUserHandler: Send + Sync {
    async fn handle(&self, command: CreateUser) -> Result<User, ApplicationError>;
}
