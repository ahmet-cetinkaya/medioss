use super::set_completed::SetCompleted;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait SetCompletedHandler: Send + Sync {
    async fn handle(&self, command: SetCompleted) -> Result<(), ApplicationError>;
}
