use chrono::{TimeZone, Utc};
use kernel::UserId;

use crate::{
    commands::{create_mood_entry, update_mood_entry},
    ports::MoodEntryRepository,
    queries::{get_mood_statistics, MoodRange, MAX_CUSTOM_RANGE_DAYS},
    test_support::{FakeMoodEntryRepository, FixedClock},
};

fn user() -> UserId {
    UserId::new(1)
}

#[tokio::test]
async fn entries_are_normalized_and_analytics_use_daily_means() {
    let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
    let entries = FakeMoodEntryRepository::default();
    let clock = FixedClock(now);
    let first = create_mood_entry(&entries, &clock, user(), 2, Some("  quiet  "))
        .await
        .unwrap();
    create_mood_entry(&entries, &clock, user(), 4, None)
        .await
        .unwrap();
    let updated = update_mood_entry(&entries, &clock, user(), first.id(), 3, Some("updated"))
        .await
        .unwrap();

    assert_eq!(updated.recorded_at(), first.recorded_at());
    let statistics = get_mood_statistics(&entries, user(), MoodRange::AllTime, 0, now)
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
    let entries = FakeMoodEntryRepository::default();
    let clock = FixedClock(now);

    assert!(create_mood_entry(&entries, &clock, user(), 0, None)
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
    let entries = FakeMoodEntryRepository::default();
    for (at, score) in [
        (Utc.with_ymd_and_hms(2026, 9, 9, 23, 0, 0).unwrap(), 1),
        (Utc.with_ymd_and_hms(2026, 9, 10, 0, 30, 0).unwrap(), 2),
        (Utc.with_ymd_and_hms(2026, 9, 15, 12, 0, 0).unwrap(), 3),
        (Utc.with_ymd_and_hms(2026, 9, 20, 23, 30, 0).unwrap(), 4),
        (Utc.with_ymd_and_hms(2026, 9, 21, 0, 30, 0).unwrap(), 5),
    ] {
        create_mood_entry(&entries, &FixedClock(at), user(), score, None)
            .await
            .unwrap();
    }
    let range = MoodRange::custom(day(2026, 9, 10), day(2026, 9, 20)).unwrap();

    let statistics = get_mood_statistics(&entries, user(), range, 0, now)
        .await
        .unwrap();

    assert_eq!(statistics.entry_count, 3, "first and last day both count");
    assert_eq!(statistics.distribution, [0, 1, 1, 1, 0]);
}

#[tokio::test]
async fn custom_range_never_reaches_past_today() {
    let now = Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap();
    let entries = FakeMoodEntryRepository::default();
    create_mood_entry(&entries, &FixedClock(now), user(), 3, None)
        .await
        .unwrap();
    let range = MoodRange::custom(day(2026, 9, 28), day(2026, 10, 15)).unwrap();

    let statistics = get_mood_statistics(&entries, user(), range, 0, now)
        .await
        .unwrap();

    assert_eq!(statistics.entry_count, 1);
}
