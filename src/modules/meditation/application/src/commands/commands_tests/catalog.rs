use super::super::*;
use super::helpers::*;
use crate::ports::{CoverNormalizationError, ProgramRepository};
use crate::test_support::{
    FailingCoverValidator, FakeConverter, FakeCoverValidator, FakeValidator,
};

#[tokio::test]
async fn category_names_must_be_unique_and_nonempty() {
    let deps = Deps::new();
    create_category(&deps.categories, &deps.clock, "Morning")
        .await
        .expect("first");
    let duplicate = create_category(&deps.categories, &deps.clock, "Morning").await;
    assert_eq!(duplicate.unwrap_err(), ApplicationError::DuplicateCategory);
    let empty = create_category(&deps.categories, &deps.clock, "   ").await;
    assert!(matches!(empty, Err(ApplicationError::InvalidInput(_))));
}

#[tokio::test]
async fn renaming_to_a_name_inserted_after_the_duplicate_check_is_a_duplicate() {
    let deps = Deps::new();
    let renamed = create_category(&deps.categories, &deps.clock, "Morning")
        .await
        .expect("renamed category");
    create_category(&deps.categories, &deps.clock, "Evening")
        .await
        .expect("existing category");

    let error = rename_category(&deps.categories, &deps.clock, renamed.id, "Evening")
        .await
        .expect_err("repository duplicate maps to application error");

    assert_eq!(error, ApplicationError::DuplicateCategory);
}

#[tokio::test]
async fn deleting_a_category_leaves_its_programs_uncategorized() {
    let deps = Deps::new();
    let category = create_category(&deps.categories, &deps.clock, "Evening")
        .await
        .expect("category");
    let program = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Wind-down",
        None,
        None,
    )
    .await
    .expect("program");
    set_program_category(
        &deps.programs,
        &deps.categories,
        program.id,
        Some(category.id),
    )
    .await
    .expect("assign");

    delete_category(&deps.categories, &deps.programs, category.id)
        .await
        .expect("delete category");

    let stored = deps
        .programs
        .find_by_id(program.id)
        .await
        .unwrap()
        .expect("program survives");
    assert_eq!(stored.category_id, None);
}

#[tokio::test]
async fn deleting_a_program_keeps_meditations_and_cleans_cover_and_members() {
    let deps = Deps::new();
    let png = vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0, 0, 0, 0, 0];
    let program = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Covered",
        Some("desc"),
        Some(png),
    )
    .await
    .expect("program with cover");
    // Uploaded directly into the program being deleted, so its file actually
    // lives in that program's folder — exercising the relocation branch,
    // not the "stayed in a different program's folder" no-op path.
    let meditation = upload_meditation(
        &deps.meditations,
        &deps.memberships,
        &deps.programs,
        &FakeValidator,
        &FakeConverter { fail: false },
        &deps.store,
        &deps.clock,
        "Home",
        program.id,
        b"OggS....OpusHead".to_vec(),
        None,
    )
    .await
    .expect("home upload");
    let original_key = meditation.storage_key().to_owned();
    assert!(original_key.starts_with(&format!("{}/", program.folder)));

    delete_program(
        &deps.programs,
        &deps.memberships,
        &deps.meditations,
        &deps.store,
        program.id,
    )
    .await
    .expect("delete program");

    let relocated_key = {
        let survivors = deps.meditations.0.meditations.lock().unwrap();
        let relocated = survivors
            .iter()
            .find(|row| row.id() == meditation.id())
            .expect("meditation row survives");
        assert_ne!(
            relocated.storage_key(),
            original_key,
            "file must be relocated"
        );
        assert!(relocated.storage_key().starts_with("_unfiled/"));
        relocated.storage_key().to_owned()
    };
    let memberships = deps.memberships.0.memberships.lock().unwrap();
    assert!(
        memberships.iter().all(|m| m.program_id != program.id),
        "no membership may reference the deleted program"
    );
    drop(memberships);
    // The cover blob is gone; the relocated audio blob remains (moved, not
    // duplicated).
    assert_eq!(deps.store.len(), 1);
    assert!(deps.store.contains_key(&relocated_key));
}

#[tokio::test]
async fn covers_are_validated_and_capped() {
    let deps = Deps::new();
    let garbage = b"definitely not an image".to_vec();
    let result = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Bad cover",
        None,
        Some(garbage),
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());

    let mut oversized = vec![0u8; MAX_COVER_BYTES + 12];
    oversized[0] = 0x89;
    oversized[1] = b'P';
    oversized[2] = b'N';
    oversized[3] = b'G';
    let result = create_program(
        &deps.programs,
        &FakeCoverValidator,
        &deps.store,
        &deps.clock,
        "Big cover",
        None,
        Some(oversized),
    )
    .await;
    assert_eq!(result.unwrap_err(), ApplicationError::PayloadTooLarge);
}

#[tokio::test]
async fn cover_normalization_failures_keep_their_operational_meaning() {
    // Invalid covers stay a client error; worker failure and busy are
    // server-side conditions, and oversize output keeps 413 semantics.
    let cases = [
        (
            CoverNormalizationError::InvalidPayload,
            ApplicationError::UnsupportedMedia,
        ),
        (
            CoverNormalizationError::OutputTooLarge,
            ApplicationError::PayloadTooLarge,
        ),
        (
            CoverNormalizationError::WorkerFailed,
            ApplicationError::Storage("cover processing failed".to_owned()),
        ),
        (CoverNormalizationError::Busy, ApplicationError::Busy),
    ];
    for (error, expected) in cases {
        let deps = Deps::new();
        let result = create_program(
            &deps.programs,
            &FailingCoverValidator(error),
            &deps.store,
            &deps.clock,
            "Cover mapping",
            None,
            Some(b"RIFF....WEBP....".to_vec()),
        )
        .await;
        assert_eq!(result.unwrap_err(), expected, "{error:?}");
        assert!(deps.store.is_empty());
        assert!(deps.programs.0.programs.lock().unwrap().is_empty());
    }
}
