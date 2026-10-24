use kernel::UserId;

use super::statistics_range::StatisticsRange;

#[derive(Debug, Clone)]
pub struct GetUserStatisticsQuery {
    pub user_id: UserId,
    pub range: StatisticsRange,
    pub tz_offset_minutes: i16,
}
