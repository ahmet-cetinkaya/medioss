use async_trait::async_trait;
use kernel::UserId;

/// One user's playback progress for one meditation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackState {
    pub position_ms: i64,
    pub completed: bool,
}

/// Persists per-user playback state.
#[async_trait]
pub trait PlaybackStateRepository: Send + Sync {
    /// Inserts or updates the stored position for (user, meditation).
    async fn upsert_position(
        &self,
        user_id: UserId,
        meditation_id: i64,
        position_ms: i64,
    ) -> Result<(), String>;

    /// Sets the completed flag for (user, meditation).
    async fn set_completed(
        &self,
        user_id: UserId,
        meditation_id: i64,
        completed: bool,
    ) -> Result<(), String>;

    /// Resets the stored position to zero for (user, meditation).
    async fn reset_position(&self, user_id: UserId, meditation_id: i64) -> Result<(), String>;

    /// Reads the playback state, if the user ever played this meditation.
    async fn get(
        &self,
        user_id: UserId,
        meditation_id: i64,
    ) -> Result<Option<PlaybackState>, String>;

    /// Every meditation this user has touched, most recently touched first,
    /// paired with the touch timestamp — feeds "recently experienced" and
    /// the history timeline's date grouping. Callers dedupe by program and
    /// cap the count themselves.
    async fn list_touched(
        &self,
        user_id: UserId,
    ) -> Result<Vec<(i64, chrono::DateTime<chrono::Utc>)>, String>;

    /// Deletes the playback state of one user (account deletion path).
    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String>;
}
