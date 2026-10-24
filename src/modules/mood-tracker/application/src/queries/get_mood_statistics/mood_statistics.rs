use chrono::NaiveDate;

/// One day of a mood trend: the day and the mean of that day's scores.
#[derive(Debug, Clone, PartialEq)]
pub struct MoodTrendPoint {
    pub date: NaiveDate,
    pub average_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoodStatistics {
    pub entry_count: usize,
    pub average_score: Option<f64>,
    pub distribution: [usize; 5],
    pub trend: Vec<MoodTrendPoint>,
}
