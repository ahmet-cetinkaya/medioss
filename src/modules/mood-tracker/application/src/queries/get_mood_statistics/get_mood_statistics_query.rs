use kernel::UserId;

use crate::models::MoodRange;

/// Aggregate one user's mood statistics over a local-date range.
#[derive(Debug, Clone)]
pub struct GetMoodStatisticsQuery {
    pub user_id: UserId,
    pub range: MoodRange,
    pub tz_offset_minutes: i16,
}
