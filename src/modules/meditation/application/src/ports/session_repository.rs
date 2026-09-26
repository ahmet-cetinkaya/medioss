use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;

/// One completed playback pass, recorded when playback reaches the end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeditationSession {
    pub id: i64,
    pub user_id: UserId,
    pub meditation_id: i64,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

/// Persists meditation sessions. Recording is idempotent per
/// (user, meditation, started_at): a retried completion report must not
/// create a second session (spec FR-021).
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Records the session, or returns the existing one when
    /// (user, meditation, started_at) already exists.
    async fn record_if_new(
        &self,
        user_id: UserId,
        meditation_id: i64,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    ) -> Result<MeditationSession, String>;

    /// How many sessions the user has for this meditation.
    async fn count_for(&self, user_id: UserId, meditation_id: i64) -> Result<i64, String>;
}
