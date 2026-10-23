use std::sync::Arc;

use super::get_user_statistics::GetUserStatistics;
use super::get_user_statistics_handler::GetUserStatisticsHandler;
use crate::ports::SessionRepository;
use std::collections::{BTreeMap, BTreeSet};

use super::statistics_day::StatisticsDay;
use super::user_statistics::UserStatistics;
use crate::ApplicationError;
use chrono::{Duration, FixedOffset, NaiveDate};

pub struct GetUserStatisticsService {
    sessions: Arc<dyn SessionRepository>,
}

impl GetUserStatisticsService {
    pub fn new(sessions: Arc<dyn SessionRepository>) -> Self {
        Self { sessions }
    }
}

#[async_trait::async_trait]
impl GetUserStatisticsHandler for GetUserStatisticsService {
    async fn handle(&self, query: GetUserStatistics) -> Result<UserStatistics, ApplicationError> {
        let GetUserStatistics {
            user_id,
            range,
            tz_offset_minutes,
            now,
        } = query;
        let offset_seconds = i32::from(tz_offset_minutes) * 60;
        let offset = FixedOffset::east_opt(offset_seconds)
            .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
        let today = now.with_timezone(&offset).date_naive();
        let range_start = range.start(today);
        let range_end = range.end(today);

        let mut all_days = BTreeSet::new();
        let mut calendar_seconds = BTreeMap::<NaiveDate, i64>::new();
        let mut total_seconds = 0_i64;

        for session in self
            .sessions
            .list_for_user(user_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            let day = session.started_at.with_timezone(&offset).date_naive();
            all_days.insert(day);
            if range_start.is_none_or(|start| day >= start) && day <= range_end {
                let seconds = session
                    .finished_at
                    .signed_duration_since(session.started_at)
                    .num_seconds()
                    .max(0);
                total_seconds += seconds;
                *calendar_seconds.entry(day).or_default() += seconds;
            }
        }

        Ok(UserStatistics {
            total_minutes: total_seconds / 60,
            total_days: calendar_seconds.len() as i64,
            current_streak: current_streak(&all_days, today),
            longest_streak: longest_streak(&all_days),
            calendar: calendar_seconds
                .into_iter()
                .map(|(date, seconds)| StatisticsDay {
                    date,
                    minutes: seconds / 60,
                })
                .collect(),
        })
    }
}

fn current_streak(days: &BTreeSet<NaiveDate>, today: NaiveDate) -> i64 {
    let yesterday = today - Duration::days(1);
    let mut day = if days.contains(&today) {
        today
    } else if days.contains(&yesterday) {
        yesterday
    } else {
        return 0;
    };
    let mut streak = 0;
    while days.contains(&day) {
        streak += 1;
        day -= Duration::days(1);
    }
    streak
}

fn longest_streak(days: &BTreeSet<NaiveDate>) -> i64 {
    let mut longest = 0;
    let mut streak = 0;
    let mut previous = None;
    for &day in days {
        if previous.is_some_and(|prior| day == prior + Duration::days(1)) {
            streak += 1;
        } else {
            streak = 1;
        }
        longest = longest.max(streak);
        previous = Some(day);
    }
    longest
}

#[cfg(test)]
#[path = "get_user_statistics_service.tests.rs"]
mod tests;
