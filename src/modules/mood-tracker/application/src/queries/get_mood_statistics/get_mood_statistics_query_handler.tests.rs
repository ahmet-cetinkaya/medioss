use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use kernel::UserId;
use mood_tracker_domain::MoodScore;

use super::GetMoodStatisticsQueryHandler;
use crate::models::MoodRange;
use crate::ports::{MoodEntryRepository, QueryHandler};
use crate::queries::get_mood_statistics::GetMoodStatisticsQuery;
use crate::test_support::{
    FailingMoodEntryRepository, FakeMoodEntryRepository, FixedClock, REPOSITORY_FAILURE,
};
use crate::ApplicationError;

const USER: i64 = 1;
const OTHER_USER: i64 = 2;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap()
}

fn at(month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, month, day, hour, minute, 0)
        .unwrap()
}

fn day(month: u32, day_of_month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day_of_month).unwrap()
}

async fn seed(entries: &FakeMoodEntryRepository, user: i64, score: u8, when: DateTime<Utc>) {
    entries
        .insert(
            UserId::new(user),
            MoodScore::new(score).unwrap(),
            None,
            when,
        )
        .await
        .unwrap();
}

async fn statistics(
    entries: &Arc<FakeMoodEntryRepository>,
    range: MoodRange,
) -> Result<crate::queries::get_mood_statistics::MoodStatistics, ApplicationError> {
    GetMoodStatisticsQueryHandler::new(entries.clone(), Arc::new(FixedClock(now())))
        .handle(GetMoodStatisticsQuery {
            user_id: UserId::new(USER),
            range,
            tz_offset_minutes: 0,
        })
        .await
}

#[tokio::test]
async fn no_entries_gives_zero_counts_and_no_average() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    let result = statistics(&entries, MoodRange::AllTime).await.unwrap();

    assert_eq!(result.entry_count, 0);
    assert_eq!(result.average_score, None);
    assert_eq!(result.distribution, [0; 5]);
    assert!(result.trend.is_empty());
}

#[tokio::test]
async fn average_distribution_and_daily_trend_use_the_callers_entries_only() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, 2, at(9, 28, 9, 0)).await;
    seed(&entries, USER, 4, at(9, 28, 18, 0)).await;
    seed(&entries, USER, 5, at(9, 29, 8, 0)).await;
    seed(&entries, OTHER_USER, 1, at(9, 29, 8, 0)).await;

    let result = statistics(&entries, MoodRange::AllTime).await.unwrap();

    assert_eq!(result.entry_count, 3);
    assert_eq!(result.distribution, [0, 1, 0, 1, 1]);
    let average = result.average_score.unwrap();
    assert!((average - 11.0 / 3.0).abs() < 1e-9);
    let trend: Vec<_> = result
        .trend
        .iter()
        .map(|point| (point.date, point.average_score))
        .collect();
    assert_eq!(trend, [(day(9, 28), 3.0), (day(9, 29), 5.0)]);
}

#[tokio::test]
async fn same_day_entries_count_once_in_the_trend_as_their_mean() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, 2, at(9, 29, 1, 0)).await;
    seed(&entries, USER, 4, at(9, 29, 2, 0)).await;

    let result = statistics(&entries, MoodRange::AllTime).await.unwrap();

    assert_eq!(result.entry_count, 2);
    assert_eq!(result.average_score, Some(3.0));
    assert_eq!(result.trend.len(), 1);
    assert_eq!(result.trend[0].average_score, 3.0);
}

#[tokio::test]
async fn preset_range_excludes_older_entries() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, 1, at(9, 1, 12, 0)).await;
    seed(&entries, USER, 5, at(9, 28, 12, 0)).await;

    let result = statistics(&entries, MoodRange::Last7Days).await.unwrap();

    assert_eq!(result.entry_count, 1);
    assert_eq!(result.distribution, [0, 0, 0, 0, 1]);
}

#[tokio::test]
async fn custom_range_includes_both_end_dates_and_nothing_outside() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, 1, at(9, 9, 23, 0)).await;
    seed(&entries, USER, 2, at(9, 10, 0, 30)).await;
    seed(&entries, USER, 3, at(9, 15, 12, 0)).await;
    seed(&entries, USER, 4, at(9, 20, 23, 30)).await;
    seed(&entries, USER, 5, at(9, 21, 0, 30)).await;
    let range = MoodRange::custom(day(9, 10), day(9, 20)).unwrap();

    let result = statistics(&entries, range).await.unwrap();

    assert_eq!(result.entry_count, 3, "first and last day both count");
    assert_eq!(result.distribution, [0, 1, 1, 1, 0]);
}

#[tokio::test]
async fn custom_range_never_reaches_past_today() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, 3, now()).await;
    seed(&entries, USER, 5, at(9, 30, 12, 0)).await;
    let range = MoodRange::custom(day(9, 28), day(10, 15)).unwrap();

    let result = statistics(&entries, range).await.unwrap();

    assert_eq!(result.entry_count, 1);
}

#[tokio::test]
async fn timezone_offset_decides_which_local_day_an_entry_falls_on() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, 4, at(9, 28, 23, 30)).await;
    let service = GetMoodStatisticsQueryHandler::new(entries.clone(), Arc::new(FixedClock(now())));

    let shifted = service
        .handle(GetMoodStatisticsQuery {
            user_id: UserId::new(USER),
            range: MoodRange::AllTime,
            tz_offset_minutes: 120,
        })
        .await
        .unwrap();

    assert_eq!(shifted.trend[0].date, day(9, 29));
}

#[tokio::test]
async fn impossible_timezone_offset_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let service = GetMoodStatisticsQueryHandler::new(entries, Arc::new(FixedClock(now())));

    let result = service
        .handle(GetMoodStatisticsQuery {
            user_id: UserId::new(USER),
            range: MoodRange::AllTime,
            tz_offset_minutes: -1440,
        })
        .await;

    assert_eq!(
        result,
        Err(ApplicationError::InvalidInput("invalid timezone offset"))
    );
}

#[tokio::test]
async fn repository_failure_is_reported_as_a_repository_error() {
    let service = GetMoodStatisticsQueryHandler::new(
        Arc::new(FailingMoodEntryRepository),
        Arc::new(FixedClock(now())),
    );

    let result = service
        .handle(GetMoodStatisticsQuery {
            user_id: UserId::new(USER),
            range: MoodRange::AllTime,
            tz_offset_minutes: 0,
        })
        .await;

    assert_eq!(
        result,
        Err(ApplicationError::Repository(REPOSITORY_FAILURE.to_owned()))
    );
}
