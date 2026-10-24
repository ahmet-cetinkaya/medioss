use kernel::UserId;

use super::mood_range::MoodRange;

/// Aggregate one user's mood statistics over a local-date range.
#[derive(Debug, Clone)]
pub struct GetMoodStatistics {
    pub user_id: UserId,
    pub range: MoodRange,
    pub tz_offset_minutes: i16,
    pub now: chrono::DateTime<chrono::Utc>,
}
