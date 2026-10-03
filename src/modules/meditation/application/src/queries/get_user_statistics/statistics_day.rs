use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatisticsDay {
    pub date: NaiveDate,
    pub minutes: i64,
}
