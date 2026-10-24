use super::set_user_status::SetUserStatus;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait SetUserStatusHandler: Send + Sync {
    async fn handle(&self, command: SetUserStatus) -> Result<(), ApplicationError>;
}
