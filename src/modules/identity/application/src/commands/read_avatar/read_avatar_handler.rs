use super::read_avatar::ReadAvatar;
use crate::ApplicationError;
use identity_domain::AvatarRef;

#[async_trait::async_trait]
pub trait ReadAvatarHandler: Send + Sync {
    async fn handle(&self, command: ReadAvatar) -> Result<(Vec<u8>, AvatarRef), ApplicationError>;
}
