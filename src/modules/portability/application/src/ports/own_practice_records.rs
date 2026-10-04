use crate::models::PracticeSnapshot;
use async_trait::async_trait;
use kernel::UserId;

#[async_trait]
pub trait OwnPracticeRecords: Send + Sync {
    async fn list_for_user(&self, user_id: UserId) -> Result<PracticeSnapshot, String>;
}
