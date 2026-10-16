use super::super::*;
use super::helpers::*;
use crate::test_support::{FakeConverter, FakeCoverValidator, FakeValidator};

#[tokio::test]
async fn upload_rejects_blank_and_oversized_names_and_garbage_bytes() {
    let deps = Deps::new();
    let converter = FakeConverter { fail: false };

    let blank = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &converter,
        &deps.store,
        &deps.clock,
        "   ",
        deps.seed_program_id().await,
        b"OggSx".to_vec(),
        None,
    )
    .await;
    assert!(matches!(blank, Err(ApplicationError::InvalidInput(_))));

    let long_name = "x".repeat(MAX_NAME_LENGTH + 1);
    let too_long = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &converter,
        &deps.store,
        &deps.clock,
        &long_name,
        deps.seed_program_id().await,
        b"OggSx".to_vec(),
        None,
    )
    .await;
    assert!(matches!(too_long, Err(ApplicationError::InvalidInput(_))));

    let garbage = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &converter,
        &deps.store,
        &deps.clock,
        "Valid name",
        deps.seed_program_id().await,
        b"not audio at all".to_vec(),
        None,
    )
    .await;
    assert_eq!(garbage.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn upload_rejects_payloads_over_the_cap_before_anything_stored() {
    let deps = Deps::new();
    let mut oversized = vec![0u8; MAX_UPLOAD_BYTES + 1];
    oversized[0] = 0xFF;
    oversized[1] = 0xFB;
    let result = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Big",
        deps.seed_program_id().await,
        oversized,
        None,
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::PayloadTooLarge);
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn upload_failure_at_the_store_leaves_no_blob() {
    // The store itself fails at `put`, before any row exists — nothing to
    // compensate, so the only assertion is that no blob was left behind.
    let deps = Deps::new();
    *deps.store.failures.lock().unwrap() = true;
    let result = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Valid",
        deps.seed_program_id().await,
        b"OggS....OpusHead".to_vec(),
        None,
    )
    .await;
    assert!(matches!(result, Err(ApplicationError::Storage(_))));
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn upload_failure_after_store_deletes_the_orphaned_blob() {
    // The store succeeds but the membership insert fails afterward — the
    // meditation row and its blob must both be rolled back rather than left
    // as an unreachable, program-less meditation.
    let deps = Deps::new();
    *deps.memberships.0.membership_add_failures.lock().unwrap() = true;
    let result = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Valid",
        deps.seed_program_id().await,
        b"OggS....OpusHead".to_vec(),
        None,
    )
    .await;
    assert!(matches!(result, Err(ApplicationError::Repository(_))));
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_removes_row_bytes_memberships_and_notes() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let program = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Program",
        None,
        None,
    )
    .await
    .expect("program");
    add_member(
        &deps.meditations,
        &deps.programs,
        &deps.memberships,
        &deps.clock,
        program.id,
        meditation.id(),
    )
    .await
    .expect("member");
    create_note(
        &deps.meditations,
        &deps.notes,
        &deps.clock,
        user_a(),
        meditation.id(),
        "felt calm",
    )
    .await
    .expect("note");

    delete_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.notes,
        &deps.store,
        meditation.id(),
    )
    .await
    .expect("delete");

    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
    assert!(deps.memberships.0.memberships.lock().unwrap().is_empty());
    assert!(deps.notes.0.notes.lock().unwrap().is_empty());
}

#[tokio::test]
async fn delete_unknown_meditation_is_not_found() {
    let deps = Deps::new();
    let result = delete_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.notes,
        &deps.store,
        999,
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::NotFound);
}
