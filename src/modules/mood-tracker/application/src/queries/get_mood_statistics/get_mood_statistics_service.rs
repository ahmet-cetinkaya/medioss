use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use kernel::UserId;

use super::get_mood_statistics::GetMoodStatistics;
use super::get_mood_statistics_handler::GetMoodStatisticsHandler;
use super::mood_statistics::{MoodStatistics, MoodTrendPoint};
use crate::models::MoodRange;
use crate::ports::{Clock, MoodEntryRepository};
use crate::ApplicationError;

pub struct GetMoodStatisticsService {
    entries: Arc<dyn MoodEntryRepository>,
    clock: Arc<dyn Clock>,
}

impl GetMoodStatisticsService {
    pub fn new(entries: Arc<dyn MoodEntryRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { entries, clock }
    }
}

#[async_trait::async_trait]
impl GetMoodStatisticsHandler for GetMoodStatisticsService {
    async fn handle(&self, query: GetMoodStatistics) -> Result<MoodStatistics, ApplicationError> {
        get_mood_statistics(
            self.entries.as_ref(),
            query.user_id,
            query.range,
            query.tz_offset_minutes,
            self.clock.now().await,
        )
        .await
    }
}

pub(crate) async fn get_mood_statistics<R>(
    entries: &R,
    user_id: UserId,
    range: MoodRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
) -> Result<MoodStatistics, ApplicationError>
where
    R: MoodEntryRepository + ?Sized,
{
    let offset = FixedOffset::east_opt(i32::from(tz_offset_minutes) * 60)
        .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
    let today = now.with_timezone(&offset).date_naive();
    let start = range.start(today);
    let end = range.end(today);
    let mut distribution = [0; 5];
    let mut by_day: BTreeMap<NaiveDate, (usize, usize)> = BTreeMap::new();
    let mut total = 0_usize;
    for entry in entries
        .list_all_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        let day = entry.recorded_at().with_timezone(&offset).date_naive();
        if day > end || start.is_some_and(|start| day < start) {
            continue;
        }
        let score = usize::from(entry.score().value());
        total += score;
        distribution[score - 1] += 1;
        let day_values = by_day.entry(day).or_default();
        day_values.0 += score;
        day_values.1 += 1;
    }
    let entry_count = distribution.iter().sum();
    Ok(MoodStatistics {
        entry_count,
        average_score: (entry_count > 0).then(|| total as f64 / entry_count as f64),
        distribution,
        trend: by_day
            .into_iter()
            .map(|(date, (sum, count))| MoodTrendPoint {
                date,
                average_score: sum as f64 / count as f64,
            })
            .collect(),
    })
}
