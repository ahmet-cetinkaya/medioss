use crate::models::MoodRecord;
use async_trait::async_trait;
use kernel::UserId;

#[async_trait]
pub trait MoodRecordSink: Send + Sync {
    async fn insert(&self, user_id: UserId, record: MoodRecord) -> Result<(), String>;
}
