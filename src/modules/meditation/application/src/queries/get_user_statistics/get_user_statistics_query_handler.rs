use std::sync::Arc;

use super::get_user_statistics_query::GetUserStatisticsQuery;
use super::get_user_statistics_query_validator::GetUserStatisticsQueryValidator;
use crate::ports::QueryHandler;
use crate::ports::{Clock, SessionRepository};
use std::collections::{BTreeMap, BTreeSet};

use super::statistics_day::StatisticsDay;
use super::user_statistics::UserStatistics;
use crate::ApplicationError;
use chrono::{Duration, NaiveDate};

pub struct GetUserStatisticsQueryHandler {
    sessions: Arc<dyn SessionRepository>,
    clock: Arc<dyn Clock>,
    validator: GetUserStatisticsQueryValidator,
}

impl GetUserStatisticsQueryHandler {
    pub fn new(sessions: Arc<dyn SessionRepository>, clock: Arc<dyn Clock>) -> Self {
        Self {
            sessions,
            clock,
            validator: GetUserStatisticsQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<GetUserStatisticsQuery> for GetUserStatisticsQueryHandler {
    type Output = UserStatistics;

    async fn handle(
        &self,
        query: GetUserStatisticsQuery,
    ) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        let GetUserStatisticsQuery {
            user_id,
            range,
            tz_offset_minutes,
        } = query;
        let offset = self.validator.timezone_offset(tz_offset_minutes)?;
        let now = self.clock.now().await;
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
#[path = "get_user_statistics_query_handler.tests.rs"]
mod tests;
