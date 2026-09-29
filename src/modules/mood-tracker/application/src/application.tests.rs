use chrono::{TimeZone, Utc};
use kernel::UserId;

use crate::{
    commands::{create_mood_entry, update_mood_entry},
    ports::MoodEntryRepository,
    queries::{get_mood_statistics, MoodRange},
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
