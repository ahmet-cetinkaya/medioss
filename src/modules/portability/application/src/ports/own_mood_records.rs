use crate::models::MoodRecord;
use async_trait::async_trait;
use kernel::UserId;

#[async_trait]
pub trait OwnMoodRecords: Send + Sync {
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<MoodRecord>, String>;
}
