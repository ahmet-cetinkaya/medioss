use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use kernel::UserId;

use super::CreateMoodEntryCommandHandler;
use crate::commands::create_mood_entry::CreateMoodEntryCommand;
use crate::ports::{CommandHandler, MoodEntryRepository};
use crate::test_support::{
    FailingMoodEntryRepository, FakeMoodEntryRepository, FixedClock, REPOSITORY_FAILURE,
};
use crate::ApplicationError;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 29, 12, 0, 0).unwrap()
}

fn command(score: u8, note: Option<&str>) -> CreateMoodEntryCommand {
    CreateMoodEntryCommand {
        user_id: UserId::new(1),
        score,
        note: note.map(str::to_owned),
    }
}

fn service(entries: &Arc<FakeMoodEntryRepository>) -> CreateMoodEntryCommandHandler {
    CreateMoodEntryCommandHandler::new(entries.clone(), Arc::new(FixedClock(now())))
}

#[tokio::test]
async fn creates_an_entry_stamped_with_the_clock_and_a_trimmed_note() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    let entry = service(&entries)
        .handle(command(4, Some("  calm  ")))
        .await
        .unwrap();

    assert_eq!(entry.score().value(), 4);
    assert_eq!(entry.note(), Some("calm"));
    assert_eq!(entry.recorded_at(), now());
    assert_eq!(entry.user_id(), UserId::new(1));
    assert_eq!(
        entries
            .list_all_for_user(UserId::new(1))
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn blank_note_is_stored_as_no_note() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    let entry = service(&entries)
        .handle(command(3, Some("   ")))
        .await
        .unwrap();

    assert_eq!(entry.note(), None);
}

#[tokio::test]
async fn scores_outside_one_to_five_are_rejected_and_nothing_is_stored() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    for score in [0, 6, 255] {
        assert_eq!(
            service(&entries).handle(command(score, None)).await,
            Err(ApplicationError::InvalidInput(
                "mood score must be between 1 and 5"
            )),
            "score {score}"
        );
    }
    assert!(entries
        .list_all_for_user(UserId::new(1))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn too_long_note_is_rejected() {
    let entries = Arc::new(FakeMoodEntryRepository::default());
    let note = "x".repeat(10_001);

    let result = service(&entries).handle(command(3, Some(&note))).await;

    assert!(matches!(result, Err(ApplicationError::InvalidInput(_))));
    assert!(entries
        .list_all_for_user(UserId::new(1))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn entries_belong_to_the_creating_user() {
    let entries = Arc::new(FakeMoodEntryRepository::default());

    service(&entries).handle(command(5, None)).await.unwrap();

    assert!(entries
        .list_all_for_user(UserId::new(2))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn repository_failure_is_reported_as_a_repository_error() {
    let service = CreateMoodEntryCommandHandler::new(
        Arc::new(FailingMoodEntryRepository),
        Arc::new(FixedClock(now())),
    );

    assert_eq!(
        service.handle(command(3, None)).await,
        Err(ApplicationError::Repository(REPOSITORY_FAILURE.to_owned()))
    );
}
