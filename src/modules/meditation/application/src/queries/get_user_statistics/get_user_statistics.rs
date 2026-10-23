use chrono::{DateTime, Utc};
use kernel::UserId;

use super::statistics_range::StatisticsRange;
#[derive(Debug, Clone)]
pub struct GetUserStatistics {
    pub user_id: UserId,
    pub range: StatisticsRange,
    pub tz_offset_minutes: i16,
    pub now: DateTime<Utc>,
}
