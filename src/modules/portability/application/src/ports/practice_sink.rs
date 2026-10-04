use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;

#[async_trait]
pub trait PracticeSink: Send + Sync {
    async fn record_session(
        &self,
        user_id: UserId,
        meditation_id: i64,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    ) -> Result<(), String>;
    async fn insert_note(
        &self,
        user_id: UserId,
        meditation_id: i64,
        text: &str,
        created_at: DateTime<Utc>,
    ) -> Result<(), String>;
    async fn upsert_position(
        &self,
        user_id: UserId,
        meditation_id: i64,
        position_ms: i64,
    ) -> Result<(), String>;
    async fn set_completed(
        &self,
        user_id: UserId,
        meditation_id: i64,
        completed: bool,
    ) -> Result<(), String>;
}
