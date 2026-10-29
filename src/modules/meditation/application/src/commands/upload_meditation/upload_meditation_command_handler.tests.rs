use crate::ports::{CommandHandler, StagedMedia};
use std::sync::Arc;

use crate::commands::upload_meditation::{UploadMeditationCommand, UploadMeditationCommandHandler};
use crate::commands::MAX_NAME_LENGTH;
use crate::test_support::{FakeConverter, FakeValidator, TestDeps, TEST_MAX_UPLOAD_BYTES};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> UploadMeditationCommandHandler {
    UploadMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        Arc::new(FakeValidator),
        Arc::new(FakeConverter { fail: false }),
        deps.store.clone(),
        deps.staging.clone(),
        deps.clock.clone(),
        TEST_MAX_UPLOAD_BYTES,
    )
}

fn upload(deps: &TestDeps, name: &str, program_id: i64, bytes: &[u8]) -> UploadMeditationCommand {
    UploadMeditationCommand {
        name: name.to_owned(),
        program_id,
        media: deps.staging.stage(bytes),
        description: None,
    }
}

#[tokio::test]
async fn upload_rejects_blank_and_oversized_names_and_garbage_bytes() {
    let deps = TestDeps::new();
    let service = service(&deps);

    let blank = service
        .handle(upload(
            &deps,
            "   ",
            deps.seed_program("Holder Program").await,
            b"OggSx",
        ))
        .await;
    assert!(matches!(blank, Err(ApplicationError::InvalidInput(_))));

    let long_name = "x".repeat(MAX_NAME_LENGTH + 1);
    let too_long = service
        .handle(upload(
            &deps,
            &long_name,
            deps.seed_program("Holder Program").await,
            b"OggSx",
        ))
        .await;
    assert!(matches!(too_long, Err(ApplicationError::InvalidInput(_))));

    let garbage = service
        .handle(upload(
            &deps,
            "Valid name",
            deps.seed_program("Holder Program").await,
            b"not audio at all",
        ))
        .await;
    assert_eq!(garbage.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn upload_rejects_payloads_over_the_cap_before_anything_stored() {
    let deps = TestDeps::new();
    let oversized = StagedMedia {
        address: "never-read".to_owned(),
        size_bytes: TEST_MAX_UPLOAD_BYTES as u64 + 1,
    };
    let result = service(&deps)
        .handle(UploadMeditationCommand {
            media: oversized,
            ..upload(
                &deps,
                "Big",
                deps.seed_program("Holder Program").await,
                b"x",
            )
        })
        .await;
    assert_eq!(result.unwrap_err(), ApplicationError::PayloadTooLarge);
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn upload_failure_at_the_store_leaves_no_blob() {
    // The store itself fails at `put`, before any row exists — nothing to
    // compensate, so the only assertion is that no blob was left behind.
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    *deps.store.failures.lock().unwrap() = true;
    let result = service(&deps)
        .handle(upload(&deps, "Valid", program_id, b"OggS....OpusHead"))
        .await;
    assert!(matches!(result, Err(ApplicationError::Storage(_))));
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn upload_failure_after_store_deletes_the_orphaned_blob() {
    // The store succeeds but the membership insert fails afterward — the
    // meditation row and its blob must both be rolled back rather than left
    // as an unreachable, program-less meditation.
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    *deps.memberships.0.membership_add_failures.lock().unwrap() = true;
    let result = service(&deps)
        .handle(upload(&deps, "Valid", program_id, b"OggS....OpusHead"))
        .await;
    assert!(matches!(result, Err(ApplicationError::Repository(_))));
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn upload_moves_the_staged_file_into_the_store() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    let command = upload(&deps, "Calm", program_id, b"OggS....OpusHead");
    assert_eq!(deps.staging.file_count(), 1);

    service(&deps).handle(command).await.unwrap();

    assert_eq!(deps.store.len(), 1);
    assert_eq!(deps.staging.file_count(), 0, "staged file is consumed");
    assert_eq!(deps.staging.deleted_count(), 0, "moved, not deleted");
}

#[tokio::test]
async fn unsupported_media_discards_the_staged_file_and_stores_nothing() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    let command = upload(&deps, "Junk", program_id, b"not audio at all");

    let result = service(&deps).handle(command).await;

    assert_eq!(result.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());
    assert_eq!(deps.staging.file_count(), 0);
    assert_eq!(deps.staging.deleted_count(), 1);
}

#[tokio::test]
async fn oversized_media_is_rejected_from_its_declared_size_and_discarded() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    let staged = deps.staging.stage(b"OggS....OpusHead");
    let command = UploadMeditationCommand {
        media: StagedMedia {
            size_bytes: TEST_MAX_UPLOAD_BYTES as u64 + 1,
            ..staged
        },
        ..upload(&deps, "Big", program_id, b"x")
    };

    let result = service(&deps).handle(command).await;

    assert_eq!(result.unwrap_err(), ApplicationError::PayloadTooLarge);
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn conversion_failure_discards_the_staged_file() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    let handler = UploadMeditationCommandHandler::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        Arc::new(FakeValidator),
        Arc::new(FakeConverter { fail: true }),
        deps.store.clone(),
        deps.staging.clone(),
        deps.clock.clone(),
        TEST_MAX_UPLOAD_BYTES,
    );
    let command = upload(&deps, "Broken", program_id, b"OggS....OpusHead");

    let result = handler.handle(command).await;

    assert_eq!(result.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert_eq!(deps.staging.file_count(), 0);
    assert_eq!(deps.staging.deleted_count(), 1);
}

#[tokio::test]
async fn store_failure_discards_the_staged_file() {
    let deps = TestDeps::new();
    let program_id = deps.seed_program("Holder Program").await;
    *deps.store.failures.lock().unwrap() = true;
    let command = upload(&deps, "Valid", program_id, b"OggS....OpusHead");

    let result = service(&deps).handle(command).await;

    assert!(matches!(result, Err(ApplicationError::Storage(_))));
    assert_eq!(deps.staging.file_count(), 0);
}
