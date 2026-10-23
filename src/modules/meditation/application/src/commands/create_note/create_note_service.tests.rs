use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::commands::create_note::{CreateNote, CreateNoteHandler, CreateNoteService};
use crate::commands::delete_note::{DeleteNote, DeleteNoteHandler, DeleteNoteService};
use crate::commands::update_note::{UpdateNote, UpdateNoteHandler, UpdateNoteService};
use crate::queries::list_notes::{ListNotes, ListNotesHandler, ListNotesService};
use crate::test_support::{user_a, user_b, FixedClock, TestDeps};
use crate::ApplicationError;

#[tokio::test]
async fn notes_are_author_scoped_and_newest_first() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    CreateNoteService::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        Arc::new(FixedClock(Utc::now() - Duration::seconds(60))),
    )
    .handle(CreateNote {
        user_id: user_a(),
        meditation_id: meditation.id(),
        text: "older".into(),
    })
    .await
    .expect("first note");
    CreateNoteService::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNote {
        user_id: user_a(),
        meditation_id: meditation.id(),
        text: "newer".into(),
    })
    .await
    .expect("second note");

    let list = ListNotesService::new(deps.meditations.clone(), deps.notes.clone());
    let listed = list
        .handle(ListNotes {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .expect("list");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].text, "newer");

    let b_listed = list
        .handle(ListNotes {
            user_id: user_b(),
            meditation_id: meditation.id(),
        })
        .await
        .expect("b list");
    assert!(b_listed.is_empty());

    // B cannot update or delete A's note: indistinguishable from unknown.
    let a_note_id = listed[0].id;
    let update = UpdateNoteService::new(deps.notes.clone(), deps.clock.clone());
    let updated = update
        .handle(UpdateNote {
            note_id: a_note_id,
            meditation_id: meditation.id(),
            user_id: user_b(),
            text: "hijack".into(),
        })
        .await;
    assert_eq!(updated.unwrap_err(), ApplicationError::NotFound);
    let deleted = DeleteNoteService::new(deps.notes.clone())
        .handle(DeleteNote {
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
        .handle(UpdateNote {
            note_id: a_note_id,
            meditation_id: other_meditation.id(),
            user_id: user_a(),
            text: "still hijack".into(),
        })
        .await;
    assert_eq!(wrong_meditation.unwrap_err(), ApplicationError::NotFound);
}
