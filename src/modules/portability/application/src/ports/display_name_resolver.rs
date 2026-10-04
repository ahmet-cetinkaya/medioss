use async_trait::async_trait;
use kernel::UserId;

#[async_trait]
pub trait DisplayNameResolver: Send + Sync {
    async fn display_name(&self, user_id: UserId) -> Result<Option<String>, String>;
}
