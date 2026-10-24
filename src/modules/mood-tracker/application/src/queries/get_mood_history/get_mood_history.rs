use kernel::UserId;

use crate::queries::get_mood_statistics::MoodRange;

/// Read one user's mood history, newest first, page-limited by a keyset
/// cursor of `(recorded_at, id)`.
#[derive(Debug, Clone)]
pub struct MoodHistory {
    pub user_id: UserId,
    pub range: MoodRange,
    pub tz_offset_minutes: i16,
    pub now: chrono::DateTime<chrono::Utc>,
    pub limit: u16,
    pub cursor: Option<(chrono::DateTime<chrono::Utc>, i64)>,
}
