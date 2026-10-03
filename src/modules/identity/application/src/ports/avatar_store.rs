use async_trait::async_trait;
use identity_domain::AvatarRef;
use kernel::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AvatarStoreError {
    #[error("avatar payload is invalid")]
    InvalidPayload,
    #[error("avatar storage operation failed")]
    Storage,
}

#[async_trait]
pub trait AvatarStore: Send + Sync {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<(Vec<u8>, &'static str), AvatarStoreError>;
    async fn write(
        &self,
        id: UserId,
        avatar: AvatarRef,
        bytes: &[u8],
    ) -> Result<(), AvatarStoreError>;
    async fn read(&self, id: UserId, avatar: AvatarRef) -> Result<Vec<u8>, AvatarStoreError>;
    async fn remove(&self, id: UserId, avatar: AvatarRef) -> Result<(), AvatarStoreError>;
}
