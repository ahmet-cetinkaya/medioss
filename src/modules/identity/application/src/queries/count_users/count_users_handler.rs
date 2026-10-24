use super::count_users::CountUsers;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait CountUsersHandler: Send + Sync {
    async fn handle(&self, query: CountUsers) -> Result<u64, ApplicationError>;
}
