use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use kernel::UserId;
use mood_tracker_domain::MoodScore;

use super::UpdateMoodEntryService;
use crate::commands::update_mood_entry::{UpdateMoodEntry, UpdateMoodEntryHandler};
use crate::ports::MoodEntryRepository;
use crate::test_support::{
    FailingMoodEntryRepository, FakeMoodEntryRepository, FixedClock, REPOSITORY_FAILURE,
};
use crate::ApplicationError;

const OWNER: i64 = 1;
const OTHER_USER: i64 = 2;

fn created_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 1, 8, 0, 0).unwrap()
}

fn edited_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap()
}

async fn seed(entries: &FakeMoodEntryRepository) -> i64 {
    entries
        .insert(
            UserId::new(OWNER),
            MoodScore::new(2).unwrap(),
            Some("before".to_owned()),
            created_at(),
        )
        .await
        .unwrap()
        .id()
}

fn command(user: i64, entry_id: i64, score: u8, note: Option<&str>) -> UpdateMoodEntry {
    UpdateMoodEntry {
        user_id: UserId::new(user),
        entry_id,
        score,
        note: note.map(str::to_owned),
    }
}

fn service(entries: &Arc<FakeMoodEntryRepository>) -> UpdateMoodEntryService {
    UpdateMoodEntryService::new(entries.clone(), Arc::new(FixedClock(edited_at())))
}

#[tokio::test]
async fn updates_score_and_note_keeping_recorded_at() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries).await;

    let updated = service(&entries)
        .handle(command(OWNER, id, 5, Some("  after  ")))
        .await
        .unwrap();

    assert_eq!(updated.score().value(), 5);
    assert_eq!(updated.note(), Some("after"));
    assert_eq!(updated.recorded_at(), created_at());
    assert_eq!(updated.updated_at(), edited_at());
}

#[tokio::test]
async fn blank_note_clears_the_note() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries).await;

    let updated = service(&entries)
        .handle(command(OWNER, id, 2, Some(" ")))
        .await
        .unwrap();

    assert_eq!(updated.note(), None);
}

#[tokio::test]
async fn invalid_score_is_rejected_and_the_entry_is_unchanged() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries).await;

    let result = service(&entries).handle(command(OWNER, id, 9, None)).await;

    assert_eq!(
        result,
        Err(ApplicationError::InvalidInput(
            "mood score must be between 1 and 5"
        ))
    );
    let stored = entries.list_all_for_user(UserId::new(OWNER)).await.unwrap();
    assert_eq!(stored[0].score().value(), 2);
    assert_eq!(stored[0].note(), Some("before"));
}

#[tokio::test]
async fn too_long_note_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries).await;
    let note = "x".repeat(10_001);

    let result = service(&entries)
        .handle(command(OWNER, id, 3, Some(&note)))
        .await;

    assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
}

#[tokio::test]
async fn another_users_entry_is_not_found_and_unchanged() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let id = seed(&entries).await;

    let result = service(&entries)
        .handle(command(OTHER_USER, id, 5, None))
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    let stored = entries.list_all_for_user(UserId::new(OWNER)).await.unwrap();
    assert_eq!(stored[0].score().value(), 2);
}

#[tokio::test]
async fn unknown_entry_is_not_found() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    assert_eq!(
        service(&entries).handle(command(OWNER, 42, 3, None)).await,
        Err(ApplicationError::NotFound)
    );
}

#[tokio::test]
async fn repository_failure_is_reported_as_a_repository_error() {
    let service = UpdateMoodEntryService::new(
        Arc::new(FailingMoodEntryRepository),
        Arc::new(FixedClock(edited_at())),
    );

    assert_eq!(
        service.handle(command(OWNER, 1, 3, None)).await,
        Err(ApplicationError::Repository(REPOSITORY_FAILURE.to_owned()))
    );
}
