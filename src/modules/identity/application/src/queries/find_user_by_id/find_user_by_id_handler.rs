use super::find_user_by_id::FindUserById;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait FindUserByIdHandler: Send + Sync {
    async fn handle(&self, query: FindUserById) -> Result<Option<User>, ApplicationError>;
}
