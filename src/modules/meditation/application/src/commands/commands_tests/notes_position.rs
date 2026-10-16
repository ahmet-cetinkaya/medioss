use chrono::{Duration, Utc};

use super::super::*;
use super::helpers::*;
use crate::test_support::FixedClock;

#[tokio::test]
async fn notes_are_author_scoped_and_newest_first() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    create_note(
        &deps.meditations,
        &deps.notes,
        &FixedClock(Utc::now() - Duration::seconds(60)),
        user_a(),
        meditation.id(),
        "older",
    )
    .await
    .expect("first note");
    create_note(
        &deps.meditations,
        &deps.notes,
        &deps.clock,
        user_a(),
        meditation.id(),
        "newer",
    )
    .await
    .expect("second note");

    let listed =
        crate::queries::list_notes(&deps.meditations, &deps.notes, user_a(), meditation.id())
            .await
            .expect("list");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].text, "newer");

    let b_listed =
        crate::queries::list_notes(&deps.meditations, &deps.notes, user_b(), meditation.id())
            .await
            .expect("b list");
    assert!(b_listed.is_empty());

    // B cannot update or delete A's note: indistinguishable from unknown.
    let a_note_id = listed[0].id;
    let updated = update_note(
        &deps.notes,
        &deps.clock,
        a_note_id,
        meditation.id(),
        user_b(),
        "hijack",
    )
    .await;
    assert_eq!(updated.unwrap_err(), ApplicationError::NotFound);
    let deleted = delete_note(&deps.notes, a_note_id, meditation.id(), user_b()).await;
    assert_eq!(deleted.unwrap_err(), ApplicationError::NotFound);

    // The author's own note, but addressed via the wrong meditation id, is
    // also NotFound: the nested route's meditation segment is enforced, not
    // decorative.
    let other_meditation = deps.seeded_meditation().await;
    let wrong_meditation = update_note(
        &deps.notes,
        &deps.clock,
        a_note_id,
        other_meditation.id(),
        user_a(),
        "still hijack",
    )
    .await;
    assert_eq!(wrong_meditation.unwrap_err(), ApplicationError::NotFound);
}

#[tokio::test]
async fn position_reports_are_validated_and_default_to_zero() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let unknown = record_position(&deps.meditations, &deps.playback, user_a(), 999, 1_000).await;
    assert_eq!(unknown.unwrap_err(), ApplicationError::NotFound);

    let negative = record_position(
        &deps.meditations,
        &deps.playback,
        user_a(),
        meditation.id(),
        -1,
    )
    .await;
    assert!(matches!(negative, Err(ApplicationError::InvalidInput(_))));

    let fresh = crate::queries::get_playback_state(
        &deps.meditations,
        &deps.playback,
        user_b(),
        meditation.id(),
    )
    .await
    .expect("state");
    assert_eq!(fresh.position_ms, 0);
    assert!(!fresh.completed);
}
