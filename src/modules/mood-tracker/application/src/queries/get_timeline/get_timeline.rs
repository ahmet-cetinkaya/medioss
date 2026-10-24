use chrono::{DateTime, Utc};
use kernel::UserId;

use crate::queries::get_mood_statistics::MoodRange;

/// Merge mood entries and meditation events into one descending timeline.
#[derive(Debug, Clone)]
pub struct GetTimeline {
    pub user_id: UserId,
    pub range: MoodRange,
    pub tz_offset_minutes: i16,
    pub now: DateTime<Utc>,
    pub limit: u16,
    pub notes_only: bool,
}
