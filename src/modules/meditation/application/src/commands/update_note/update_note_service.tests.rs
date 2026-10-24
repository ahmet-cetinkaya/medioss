use crate::commands::create_note::{CreateNote, CreateNoteHandler, CreateNoteService};
use crate::commands::update_note::{UpdateNote, UpdateNoteHandler, UpdateNoteService};
use crate::ports::NoteRepository;
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

async fn seed_note(deps: &TestDeps, meditation_id: i64) -> i64 {
    CreateNoteService::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNote {
        user_id: user_a(),
        meditation_id,
        text: "before".into(),
    })
    .await
    .expect("seed note")
    .id
}

fn service(deps: &TestDeps) -> UpdateNoteService {
    UpdateNoteService::new(deps.notes.clone(), deps.clock.clone())
}

fn command(note_id: i64, meditation_id: i64, text: &str) -> UpdateNote {
    UpdateNote {
        note_id,
        meditation_id,
        user_id: user_a(),
        text: text.into(),
    }
}

#[tokio::test]
async fn updates_the_text_trimmed() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    service(&deps)
        .handle(command(note_id, meditation.id(), "  after  "))
        .await
        .expect("update");

    let notes = deps
        .notes
        .list_for(user_a(), meditation.id())
        .await
        .unwrap();
    assert_eq!(notes[0].text, "after");
}

#[tokio::test]
async fn blank_text_is_invalid_input_and_keeps_the_note() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    let result = service(&deps)
        .handle(command(note_id, meditation.id(), "   "))
        .await;

    assert_eq!(
        result,
        Err(ApplicationError::InvalidInput(
            "note text must not be empty"
        ))
    );
    let notes = deps
        .notes
        .list_for(user_a(), meditation.id())
        .await
        .unwrap();
    assert_eq!(notes[0].text, "before");
}

#[tokio::test]
async fn another_authors_note_is_not_found() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    let result = service(&deps)
        .handle(UpdateNote {
            user_id: user_b(),
            ..command(note_id, meditation.id(), "hijack")
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    let notes = deps
        .notes
        .list_for(user_a(), meditation.id())
        .await
        .unwrap();
    assert_eq!(notes[0].text, "before");
}

#[tokio::test]
async fn unknown_note_is_not_found() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    let result = service(&deps)
        .handle(command(9999, meditation.id(), "text"))
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn mismatched_meditation_is_not_found() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let note_id = seed_note(&deps, meditation.id()).await;

    let result = service(&deps)
        .handle(command(note_id, meditation.id() + 1, "text"))
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
