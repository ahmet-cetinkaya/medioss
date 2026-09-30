use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeditationTimelineEvent {
    pub id: i64,
    pub meditation_name: String,
    pub note: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub kind: MeditationTimelineEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeditationTimelineEventKind {
    CompletedSession,
    Note,
}

#[async_trait]
pub trait MeditationTimelineSource: Send + Sync {
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<MeditationTimelineEvent>, String>;
}
