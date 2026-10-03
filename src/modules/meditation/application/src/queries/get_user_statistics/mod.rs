mod statistics_day;
mod statistics_range;
mod user_statistics;

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use kernel::UserId;

use crate::{ports::SessionRepository, ApplicationError};

pub use self::statistics_day::StatisticsDay;
pub use self::statistics_range::StatisticsRange;
pub use self::user_statistics::UserStatistics;

pub async fn get_user_statistics<R>(
    sessions: &R,
    user_id: UserId,
    range: StatisticsRange,
    tz_offset_minutes: i16,
    now: DateTime<Utc>,
) -> Result<UserStatistics, ApplicationError>
where
    R: SessionRepository + ?Sized,
{
    let offset_seconds = i32::from(tz_offset_minutes) * 60;
    let offset = FixedOffset::east_opt(offset_seconds)
        .ok_or(ApplicationError::InvalidInput("invalid timezone offset"))?;
    let today = now.with_timezone(&offset).date_naive();
    let range_start = match range {
        StatisticsRange::Today => Some(today),
        StatisticsRange::Last30Days => Some(today - Duration::days(29)),
        StatisticsRange::AllTime => None,
    };

    let mut all_days = BTreeSet::new();
    let mut calendar_seconds = BTreeMap::<NaiveDate, i64>::new();
    let mut total_seconds = 0_i64;

    for session in sessions
        .list_for_user(user_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        let day = session.started_at.with_timezone(&offset).date_naive();
        all_days.insert(day);
        if range_start.is_none_or(|start| day >= start) && day <= today {
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
