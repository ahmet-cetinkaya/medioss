use crate::commands::create_note::{CreateNoteCommand, CreateNoteCommandHandler};
use crate::ports::CommandHandler as _;
use crate::queries::list_notes::{ListNotes, ListNotesHandler, ListNotesService};
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> ListNotesService {
    ListNotesService::new(deps.meditations.clone(), deps.notes.clone())
}

#[tokio::test]
async fn lists_only_the_requesters_notes() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    CreateNoteCommandHandler::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNoteCommand {
        user_id: user_a(),
        meditation_id: meditation.id(),
        text: "mine".into(),
    })
    .await
    .unwrap();

    let own = service(&deps)
        .handle(ListNotes {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();
    let other = service(&deps)
        .handle(ListNotes {
            user_id: user_b(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();

    assert_eq!(own.len(), 1);
    assert_eq!(own[0].text, "mine");
    assert!(other.is_empty());
}

#[tokio::test]
async fn unknown_meditation_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(ListNotes {
            user_id: user_a(),
            meditation_id: 9999,
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
