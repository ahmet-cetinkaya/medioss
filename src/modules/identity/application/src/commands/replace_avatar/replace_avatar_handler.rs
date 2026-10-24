use super::replace_avatar::ReplaceAvatar;
use crate::ApplicationError;
use identity_domain::User;

#[async_trait::async_trait]
pub trait ReplaceAvatarHandler: Send + Sync {
    async fn handle(&self, command: ReplaceAvatar) -> Result<User, ApplicationError>;
}
