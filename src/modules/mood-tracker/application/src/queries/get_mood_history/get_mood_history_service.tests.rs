use std::sync::Arc;

use chrono::{DateTime, Duration, TimeZone, Utc};
use kernel::UserId;
use mood_tracker_domain::MoodScore;

use super::GetMoodHistoryService;
use crate::models::MoodRange;
use crate::ports::MoodEntryRepository;
use crate::queries::get_mood_history::{GetMoodHistory, GetMoodHistoryHandler};
use crate::test_support::{
    FailingMoodEntryRepository, FakeMoodEntryRepository, FixedClock, REPOSITORY_FAILURE,
};
use crate::ApplicationError;

const USER: i64 = 1;
const OTHER_USER: i64 = 2;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap()
}

async fn seed(entries: &FakeMoodEntryRepository, user: i64, at: DateTime<Utc>) -> i64 {
    entries
        .insert(UserId::new(user), MoodScore::new(3).unwrap(), None, at)
        .await
        .unwrap()
        .id()
}

fn query(range: MoodRange, limit: u16) -> GetMoodHistory {
    GetMoodHistory {
        user_id: UserId::new(USER),
        range,
        tz_offset_minutes: 0,
        limit,
        cursor: None,
    }
}

fn service(entries: &Arc<FakeMoodEntryRepository>) -> GetMoodHistoryService {
    GetMoodHistoryService::new(entries.clone(), Arc::new(FixedClock(now())))
}

#[tokio::test]
async fn empty_history_is_an_empty_page_without_cursor() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    let page = service(&entries)
        .handle(query(MoodRange::AllTime, 10))
        .await
        .unwrap();

    assert!(page.items.is_empty());
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn lists_newest_first_and_only_the_callers_entries() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let older = seed(&entries, USER, now() - Duration::days(2)).await;
    let newer = seed(&entries, USER, now() - Duration::days(1)).await;
    seed(&entries, OTHER_USER, now()).await;

    let page = service(&entries)
        .handle(query(MoodRange::AllTime, 10))
        .await
        .unwrap();

    let ids: Vec<_> = page.items.iter().map(|entry| entry.id()).collect();
    assert_eq!(ids, [newer, older]);
}

#[tokio::test]
async fn range_filters_out_entries_before_its_start() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, now() - Duration::days(10)).await;
    let recent = seed(&entries, USER, now() - Duration::days(3)).await;

    let page = service(&entries)
        .handle(query(MoodRange::Last7Days, 10))
        .await
        .unwrap();

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id(), recent);
}

#[tokio::test]
async fn timezone_offset_moves_the_day_boundary() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    // 23:30 UTC on the 28th is already the 29th at UTC+2, i.e. "today".
    let late = Utc.with_ymd_and_hms(2026, 9, 28, 23, 30, 0).unwrap();
    seed(&entries, USER, late).await;

    let utc = service(&entries)
        .handle(query(MoodRange::Today, 10))
        .await
        .unwrap();
    let mut plus_two = query(MoodRange::Today, 10);
    plus_two.tz_offset_minutes = 120;
    let shifted = service(&entries).handle(plus_two).await.unwrap();

    assert!(utc.items.is_empty());
    assert_eq!(shifted.items.len(), 1);
}

#[tokio::test]
async fn pages_are_chained_by_cursor_without_gaps_or_duplicates() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    for day in 0..5 {
        seed(&entries, USER, now() - Duration::days(day)).await;
    }
    let service = service(&entries);

    let first = service.handle(query(MoodRange::AllTime, 2)).await.unwrap();
    assert_eq!(first.items.len(), 2);
    let last_of_first = first.items.last().unwrap();
    assert_eq!(
        first.next_cursor,
        Some((last_of_first.recorded_at(), last_of_first.id()))
    );

    let mut second_query = query(MoodRange::AllTime, 2);
    second_query.cursor = first.next_cursor;
    let second = service.handle(second_query).await.unwrap();
    assert_eq!(second.items.len(), 2);

    let mut third_query = query(MoodRange::AllTime, 2);
    third_query.cursor = second.next_cursor;
    let third = service.handle(third_query).await.unwrap();
    assert_eq!(third.items.len(), 1);
    assert_eq!(third.next_cursor, None);

    let mut ids: Vec<_> = [first, second, third]
        .iter()
        .flat_map(|page| page.items.iter().map(|entry| entry.id()))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 5);
}

#[tokio::test]
async fn exact_page_size_has_no_next_cursor() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    seed(&entries, USER, now()).await;
    seed(&entries, USER, now() - Duration::days(1)).await;

    let page = service(&entries)
        .handle(query(MoodRange::AllTime, 2))
        .await
        .unwrap();

    assert_eq!(page.items.len(), 2);
    assert_eq!(page.next_cursor, None);
}

#[tokio::test]
async fn page_size_outside_one_to_one_hundred_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    for limit in [0, 101] {
        assert_eq!(
            service(&entries)
                .handle(query(MoodRange::AllTime, limit))
                .await,
            Err(ApplicationError::InvalidInput(
                "page size must be between 1 and 100"
            )),
            "limit {limit}"
        );
    }
    assert!(service(&entries)
        .handle(query(MoodRange::AllTime, 100))
        .await
        .is_ok());
}

#[tokio::test]
async fn impossible_timezone_offset_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let mut bad = query(MoodRange::AllTime, 10);
    bad.tz_offset_minutes = 1440;

    assert_eq!(
        service(&entries).handle(bad).await,
        Err(ApplicationError::InvalidInput("invalid timezone offset"))
    );
}

#[tokio::test]
async fn repository_failure_is_reported_as_a_repository_error() {
    let service = GetMoodHistoryService::new(
        Arc::new(FailingMoodEntryRepository),
        Arc::new(FixedClock(now())),
    );

    assert_eq!(
        service.handle(query(MoodRange::AllTime, 10)).await,
        Err(ApplicationError::Repository(REPOSITORY_FAILURE.to_owned()))
    );
}
