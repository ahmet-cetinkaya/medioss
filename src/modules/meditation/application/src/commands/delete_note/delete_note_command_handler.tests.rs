use crate::commands::create_note::{CreateNoteCommand, CreateNoteCommandHandler};
use crate::commands::delete_note::{DeleteNoteCommand, DeleteNoteCommandHandler};
use crate::ports::CommandHandler as _;
use crate::ports::NoteRepository;
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

async fn seed_note(deps: &TestDeps, meditation_id: i64) -> i64 {
    CreateNoteCommandHandler::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNoteCommand {
        user_id: user_a(),
        meditation_id,
        text: "keep calm".into(),
    })
    .await
    .expect("seed note")
    .id
}

#[tokio::test]
async fn author_deletes_their_note() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    DeleteNoteCommandHandler::new(deps.notes.clone())
        .handle(DeleteNoteCommand {
            note_id,
            meditation_id: meditation.id(),
            user_id: user_a(),
        })
        .await
        .expect("delete");

    let remaining = deps
        .notes
        .list_for(user_a(), meditation.id())
        .await
        .unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn another_authors_note_is_not_found_and_survives() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    let result = DeleteNoteCommandHandler::new(deps.notes.clone())
        .handle(DeleteNoteCommand {
            note_id,
            meditation_id: meditation.id(),
            user_id: user_b(),
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    let remaining = deps
        .notes
        .list_for(user_a(), meditation.id())
        .await
        .unwrap();
    assert_eq!(remaining.len(), 1);
}

#[tokio::test]
async fn unknown_note_is_not_found() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    let result = DeleteNoteCommandHandler::new(deps.notes.clone())
        .handle(DeleteNoteCommand {
            note_id: 9999,
            meditation_id: meditation.id(),
            user_id: user_a(),
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn note_under_a_different_meditation_is_not_found() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    let result = DeleteNoteCommandHandler::new(deps.notes.clone())
        .handle(DeleteNoteCommand {
            note_id,
            meditation_id: meditation.id() + 1,
            user_id: user_a(),
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
