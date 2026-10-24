use super::clear_avatar::ClearAvatar;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait ClearAvatarHandler: Send + Sync {
    async fn handle(&self, command: ClearAvatar) -> Result<(), ApplicationError>;
}
