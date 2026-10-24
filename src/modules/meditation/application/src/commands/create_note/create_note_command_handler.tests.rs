use crate::ports::CommandHandler as _;
use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::commands::create_note::{CreateNoteCommand, CreateNoteCommandHandler};
use crate::commands::delete_note::{DeleteNoteCommand, DeleteNoteCommandHandler};
use crate::commands::update_note::{UpdateNoteCommand, UpdateNoteCommandHandler};
use crate::ports::QueryHandler;
use crate::queries::list_notes::{ListNotesQuery, ListNotesQueryHandler};
use crate::test_support::{user_a, user_b, FixedClock, TestDeps};
use crate::ApplicationError;

#[tokio::test]
async fn notes_are_author_scoped_and_newest_first() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    CreateNoteCommandHandler::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        Arc::new(FixedClock(Utc::now() - Duration::seconds(60))),
    )
    .handle(CreateNoteCommand {
        user_id: user_a(),
        meditation_id: meditation.id(),
        text: "older".into(),
    })
    .await
    .expect("first note");
    CreateNoteCommandHandler::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNoteCommand {
        user_id: user_a(),
        meditation_id: meditation.id(),
        text: "newer".into(),
    })
    .await
    .expect("second note");

    let list = ListNotesQueryHandler::new(deps.meditations.clone(), deps.notes.clone());
    let listed = list
        .handle(ListNotesQuery {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .expect("list");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].text, "newer");

    let b_listed = list
        .handle(ListNotesQuery {
            user_id: user_b(),
            meditation_id: meditation.id(),
        })
        .await
        .expect("b list");
    assert!(b_listed.is_empty());

    // B cannot update or delete A's note: indistinguishable from unknown.
    let a_note_id = listed[0].id;
    let update = UpdateNoteCommandHandler::new(deps.notes.clone(), deps.clock.clone());
    let updated = update
        .handle(UpdateNoteCommand {
            note_id: a_note_id,
            meditation_id: meditation.id(),
            user_id: user_b(),
            text: "hijack".into(),
        })
        .await;
    assert_eq!(updated.unwrap_err(), ApplicationError::NotFound);
    let deleted = DeleteNoteCommandHandler::new(deps.notes.clone())
        .handle(DeleteNoteCommand {
            note_id: a_note_id,
            meditation_id: meditation.id(),
            user_id: user_b(),
        })
        .await;
    assert_eq!(deleted.unwrap_err(), ApplicationError::NotFound);

    // The author's own note, but addressed via the wrong meditation id, is
    // also NotFound: the nested route's meditation segment is enforced, not
    // decorative.
    let other_meditation = deps.seeded_meditation().await;
    let wrong_meditation = update
        .handle(UpdateNoteCommand {
            note_id: a_note_id,
            meditation_id: other_meditation.id(),
            user_id: user_a(),
            text: "still hijack".into(),
        })
        .await;
    assert_eq!(wrong_meditation.unwrap_err(), ApplicationError::NotFound);
}
