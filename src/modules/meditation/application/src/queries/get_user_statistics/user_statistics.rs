use super::statistics_day::StatisticsDay;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserStatistics {
    pub total_minutes: i64,
    pub total_days: i64,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub calendar: Vec<StatisticsDay>,
}
