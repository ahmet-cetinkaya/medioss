use kernel::UserId;

use crate::models::MoodRange;

/// Merge mood entries and meditation events into one descending timeline.
#[derive(Debug, Clone)]
pub struct GetTimelineQuery {
    pub user_id: UserId,
    pub range: MoodRange,
    pub tz_offset_minutes: i16,
    pub limit: u16,
    pub notes_only: bool,
}
