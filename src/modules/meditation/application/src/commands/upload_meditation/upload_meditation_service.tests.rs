use std::sync::Arc;

use crate::commands::upload_meditation::{
    UploadMeditation, UploadMeditationHandler, UploadMeditationService, MAX_UPLOAD_BYTES,
};
use crate::commands::MAX_NAME_LENGTH;
use crate::test_support::{FakeConverter, FakeValidator, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> UploadMeditationService {
    UploadMeditationService::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
        Arc::new(FakeValidator),
        Arc::new(FakeConverter { fail: false }),
        deps.store.clone(),
        deps.clock.clone(),
    )
}

fn upload(name: &str, program_id: i64, bytes: Vec<u8>) -> UploadMeditation {
    UploadMeditation {
        name: name.to_owned(),
        program_id,
        bytes,
        description: None,
    }
}

#[tokio::test]
async fn upload_rejects_blank_and_oversized_names_and_garbage_bytes() {
    let deps = TestDeps::new();
    let service = service(&deps);

    let blank = service
        .handle(upload(
            "   ",
            deps.seed_program("Holder Program").await,
            b"OggSx".to_vec(),
        ))
        .await;
    assert!(matches!(blank, Err(ApplicationError::InvalidInput(_))));

    let long_name = "x".repeat(MAX_NAME_LENGTH + 1);
    let too_long = service
        .handle(upload(
            &long_name,
            deps.seed_program("Holder Program").await,
            b"OggSx".to_vec(),
        ))
        .await;
    assert!(matches!(too_long, Err(ApplicationError::InvalidInput(_))));

    let garbage = service
        .handle(upload(
            "Valid name",
            deps.seed_program("Holder Program").await,
            b"not audio at all".to_vec(),
        ))
        .await;
    assert_eq!(garbage.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}

#[tokio::test]
async fn upload_rejects_payloads_over_the_cap_before_anything_stored() {
    let deps = TestDeps::new();
    let mut oversized = vec![0u8; MAX_UPLOAD_BYTES + 1];
    oversized[0] = 0xFF;
    oversized[1] = 0xFB;
    let result = service(&deps)
        .handle(upload(
            "Big",
            deps.seed_program("Holder Program").await,
            oversized,
        ))
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
        .handle(upload("Valid", program_id, b"OggS....OpusHead".to_vec()))
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
        .handle(upload("Valid", program_id, b"OggS....OpusHead".to_vec()))
        .await;
    assert!(matches!(result, Err(ApplicationError::Repository(_))));
    assert!(deps.store.is_empty());
    assert!(deps.meditations.0.meditations.lock().unwrap().is_empty());
}
