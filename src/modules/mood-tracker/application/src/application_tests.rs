use chrono::{TimeZone, Utc};
use kernel::UserId;

use crate::commands::create_mood_entry::{
    CreateMoodEntry, CreateMoodEntryHandler, CreateMoodEntryService,
};
use crate::commands::update_mood_entry::{
    UpdateMoodEntry, UpdateMoodEntryHandler, UpdateMoodEntryService,
};
use crate::models::{MoodRange, MAX_CUSTOM_RANGE_DAYS};
use crate::ports::MoodEntryRepository;
use crate::queries::get_mood_statistics::{
    GetMoodStatistics, GetMoodStatisticsHandler, GetMoodStatisticsService,
};
use crate::test_support::{FakeMoodEntryRepository, FixedClock};
use std::sync::Arc;

fn user() -> UserId {
    UserId::new(1)
}

#[tokio::test]
async fn entries_are_normalized_and_analytics_use_daily_means() {
    let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let clock = Arc::new(FixedClock(now));
    let create = CreateMoodEntryService::new(entries.clone(), clock.clone());
    let update = UpdateMoodEntryService::new(entries.clone(), clock.clone());
    let first = create
        .handle(CreateMoodEntry {
            user_id: user(),
            score: 2,
            note: Some("  quiet  ".to_owned()),
        })
        .await
        .unwrap();
    create
        .handle(CreateMoodEntry {
            user_id: user(),
            score: 4,
            note: None,
        })
        .await
        .unwrap();
    let updated = update
        .handle(UpdateMoodEntry {
            user_id: user(),
            entry_id: first.id(),
            score: 3,
            note: Some("updated".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(updated.recorded_at(), first.recorded_at());
    let statistics = GetMoodStatisticsService::new(entries.clone(), Arc::new(FixedClock(now)))
        .handle(GetMoodStatistics {
            user_id: user(),
            range: MoodRange::AllTime,
            tz_offset_minutes: 0,
        })
        .await
        .unwrap();
    assert_eq!(statistics.entry_count, 2);
    assert_eq!(statistics.average_score, Some(3.5));
    assert_eq!(statistics.distribution, [0, 0, 1, 1, 0]);
    assert_eq!(statistics.trend.len(), 1);
    assert_eq!(statistics.trend[0].average_score, 3.5);
}

#[tokio::test]
async fn invalid_scores_do_not_create_entries() {
    let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let clock = Arc::new(FixedClock(now));
    let create = CreateMoodEntryService::new(entries.clone(), clock);

    assert!(create
        .handle(CreateMoodEntry {
            user_id: user(),
            score: 0,
            note: None,
        })
        .await
        .is_err());
    assert!(entries.list_all_for_user(user()).await.unwrap().is_empty());
}

fn day(year: i32, month: u32, day_of_month: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(year, month, day_of_month).unwrap()
}

#[test]
fn custom_range_accepts_a_valid_span_and_rejects_bad_ones() {
    let from = day(2026, 9, 1);
    assert_eq!(
        MoodRange::custom(from, from).unwrap(),
        MoodRange::Custom { from, to: from },
        "one day is the shortest valid range"
    );
    let last_valid = from + chrono::Duration::days(MAX_CUSTOM_RANGE_DAYS - 1);
    assert!(MoodRange::custom(from, last_valid).is_ok());
    assert!(
        MoodRange::custom(from, last_valid + chrono::Duration::days(1)).is_err(),
        "one day past the maximum span is rejected"
    );
    assert!(
        MoodRange::custom(day(2026, 9, 2), from).is_err(),
        "from after to is rejected"
    );
}

#[tokio::test]
async fn custom_range_includes_both_end_dates_and_nothing_outside() {
    let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
    let entries = Arc::new(FakeMoodEntryRepository::default());
    for (at, score) in [
        (Utc.with_ymd_and_hms(2026, 9, 9, 23, 0, 0).unwrap(), 1),
        (Utc.with_ymd_and_hms(2026, 9, 10, 0, 30, 0).unwrap(), 2),
        (Utc.with_ymd_and_hms(2026, 9, 15, 12, 0, 0).unwrap(), 3),
        (Utc.with_ymd_and_hms(2026, 9, 20, 23, 30, 0).unwrap(), 4),
        (Utc.with_ymd_and_hms(2026, 9, 21, 0, 30, 0).unwrap(), 5),
    ] {
        // one service per entry: each entry is created at its own instant
        CreateMoodEntryService::new(entries.clone(), Arc::new(FixedClock(at)))
            .handle(CreateMoodEntry {
                user_id: user(),
                score,
                note: None,
            })
            .await
            .unwrap();
    }
    let range = MoodRange::custom(day(2026, 9, 10), day(2026, 9, 20)).unwrap();

    let statistics = GetMoodStatisticsService::new(entries.clone(), Arc::new(FixedClock(now)))
        .handle(GetMoodStatistics {
            user_id: user(),
            range,
            tz_offset_minutes: 0,
        })
        .await
        .unwrap();

    assert_eq!(statistics.entry_count, 3, "first and last day both count");
    assert_eq!(statistics.distribution, [0, 1, 1, 1, 0]);
}

#[tokio::test]
async fn custom_range_never_reaches_past_today() {
    let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let create = CreateMoodEntryService::new(entries.clone(), Arc::new(FixedClock(now)));
    create
        .handle(CreateMoodEntry {
            user_id: user(),
            score: 3,
            note: None,
        })
        .await
        .unwrap();
    let range = MoodRange::custom(day(2026, 9, 28), day(2026, 10, 15)).unwrap();

    let statistics = GetMoodStatisticsService::new(entries.clone(), Arc::new(FixedClock(now)))
        .handle(GetMoodStatistics {
            user_id: user(),
            range,
            tz_offset_minutes: 0,
        })
        .await
        .unwrap();

    assert_eq!(statistics.entry_count, 1);
}
