use super::list_users::ListUsers;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait ListUsersHandler: Send + Sync {
    async fn handle(&self, query: ListUsers) -> Result<Vec<User>, ApplicationError>;
}
