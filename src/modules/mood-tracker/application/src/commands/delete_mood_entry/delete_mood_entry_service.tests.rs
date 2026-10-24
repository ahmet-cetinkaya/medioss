use std::sync::Arc;

use chrono::{TimeZone, Utc};
use kernel::UserId;
use mood_tracker_domain::MoodScore;

use super::DeleteMoodEntryService;
use crate::commands::delete_mood_entry::{DeleteMoodEntry, DeleteMoodEntryHandler};
use crate::ports::MoodEntryRepository;
use crate::test_support::{
    FailingMoodEntryRepository, FakeMoodEntryRepository, REPOSITORY_FAILURE,
};
use crate::ApplicationError;

const OWNER: i64 = 1;
const OTHER_USER: i64 = 2;

async fn seed(entries: &FakeMoodEntryRepository, user: i64) -> i64 {
    entries
        .insert(
            UserId::new(user),
            MoodScore::new(3).unwrap(),
            None,
            Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap(),
        )
        .await
        .unwrap()
        .id()
}

fn command(user: i64, entry_id: i64) -> DeleteMoodEntry {
    DeleteMoodEntry {
        user_id: UserId::new(user),
        entry_id,
    }
}

#[tokio::test]
async fn deletes_the_owners_entry() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries, OWNER).await;

    DeleteMoodEntryService::new(entries.clone())
        .handle(command(OWNER, id))
        .await
        .unwrap();

    assert!(entries
        .list_all_for_user(UserId::new(OWNER))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn another_users_entry_is_not_found_and_survives() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries, OWNER).await;

    let result = DeleteMoodEntryService::new(entries.clone())
        .handle(command(OTHER_USER, id))
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    assert_eq!(
        entries
            .list_all_for_user(UserId::new(OWNER))
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn unknown_entry_is_not_found() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    assert_eq!(
        DeleteMoodEntryService::new(entries)
            .handle(command(OWNER, 99))
            .await,
        Err(ApplicationError::NotFound)
    );
}

#[tokio::test]
async fn deleting_twice_reports_not_found_the_second_time() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries, OWNER).await;
    let service = DeleteMoodEntryService::new(entries);

    service.handle(command(OWNER, id)).await.unwrap();

    assert_eq!(
        service.handle(command(OWNER, id)).await,
        Err(ApplicationError::NotFound)
    );
}

#[tokio::test]
async fn repository_failure_is_reported_as_a_repository_error() {
    let service = DeleteMoodEntryService::new(Arc::new(FailingMoodEntryRepository));

    assert_eq!(
        service.handle(command(OWNER, 1)).await,
        Err(ApplicationError::Repository(REPOSITORY_FAILURE.to_owned()))
    );
}
