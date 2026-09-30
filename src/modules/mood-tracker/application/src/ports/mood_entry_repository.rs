use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use mood_tracker_domain::{MoodEntry, MoodScore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<(DateTime<Utc>, i64)>,
}

#[async_trait]
pub trait MoodEntryRepository: Send + Sync {
    async fn insert(
        &self,
        user_id: UserId,
        score: MoodScore,
        note: Option<String>,
        recorded_at: DateTime<Utc>,
    ) -> Result<MoodEntry, String>;

    async fn update(
        &self,
        user_id: UserId,
        entry_id: i64,
        score: MoodScore,
        note: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Result<Option<MoodEntry>, String>;

    async fn delete(&self, user_id: UserId, entry_id: i64) -> Result<bool, String>;

    async fn list_all_for_user(&self, user_id: UserId) -> Result<Vec<MoodEntry>, String>;
}
