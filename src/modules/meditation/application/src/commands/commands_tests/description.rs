use super::super::*;
use super::helpers::*;
use crate::ports::MeditationRepository;
use crate::test_support::{FakeConverter, FakeValidator};

#[tokio::test]
async fn description_is_stored_on_upload_and_updatable() {
    let deps = Deps::new();
    let program_id = deps.seed_program_id().await;
    let uploaded = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Described",
        program_id,
        b"OggS....OpusHead".to_vec(),
        Some(" hello "),
    )
    .await
    .unwrap();
    assert_eq!(uploaded.description(), Some("hello"));

    let updated = update_meditation_description(&deps.meditations, uploaded.id(), Some("new"))
        .await
        .unwrap();
    assert_eq!(updated.description(), Some("new"));
    let cleared = update_meditation_description(&deps.meditations, uploaded.id(), Some(""))
        .await
        .unwrap();
    assert_eq!(cleared.description(), None);
    let stored = deps
        .meditations
        .find_by_id(uploaded.id())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.description(), None);
}

#[tokio::test]
async fn description_update_rejects_overlong_text_and_unknown_ids() {
    let deps = Deps::new();
    let seeded = deps.seeded_meditation().await;
    let long = "a".repeat(2001);
    assert_eq!(
        update_meditation_description(&deps.meditations, seeded.id(), Some(&long)).await,
        Err(ApplicationError::InvalidDescription)
    );
    assert_eq!(
        update_meditation_description(&deps.meditations, 999, Some("x")).await,
        Err(ApplicationError::NotFound)
    );
}
