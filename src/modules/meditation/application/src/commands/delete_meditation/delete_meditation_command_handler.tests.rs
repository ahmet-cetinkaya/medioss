use crate::commands::add_member::{AddMemberCommand, AddMemberCommandHandler};
use crate::commands::create_note::{CreateNoteCommand, CreateNoteCommandHandler};
use crate::commands::delete_meditation::{DeleteMeditationCommand, DeleteMeditationCommandHandler};
use crate::ports::CommandHandler;
use crate::test_support::{user_a, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> DeleteMeditationCommandHandler {
    DeleteMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.notes.clone(),
        deps.store.clone(),
    )
}

#[tokio::test]
async fn delete_removes_row_bytes_memberships_and_notes() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let program_id = deps.seed_program("Program").await;
    AddMemberCommandHandler::new(
        deps.meditations.clone(),
        deps.programs.clone(),
        deps.memberships.clone(),
        deps.clock.clone(),
    )
    .handle(AddMemberCommand {
        program_id,
        meditation_id: meditation.id(),
    })
    .await
    .expect("member");
    CreateNoteCommandHandler::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNoteCommand {
        user_id: user_a(),
        meditation_id: meditation.id(),
        text: "felt calm".into(),
    })
    .await
    .expect("note");

    service(&deps)
        .handle(DeleteMeditationCommand {
            meditation_id: meditation.id(),
        })
        .await
        .expect("delete");

    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
    assert!(deps.memberships.0.memberships.lock().unwrap().is_empty());
    assert!(deps.notes.0.notes.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_unknown_meditation_is_not_found() {
    let deps = TestDeps::new();
    let result = service(&deps)
        .handle(DeleteMeditationCommand { meditation_id: 999 })
        .await;
    assert_eq!(result.unwrap_err(), ApplicationError::NotFound);
}
