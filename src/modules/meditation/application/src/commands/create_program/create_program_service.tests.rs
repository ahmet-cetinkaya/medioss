use std::sync::Arc;

use crate::commands::create_program::{
    CreateProgram, CreateProgramHandler, CreateProgramService, MAX_COVER_BYTES,
};
use crate::ports::{CoverNormalizationError, CoverValidator};
use crate::test_support::{FailingCoverValidator, FakeCoverValidator, TestDeps};
use crate::ApplicationError;

fn service_with(deps: &TestDeps, validator: Arc<dyn CoverValidator>) -> CreateProgramService {
    CreateProgramService::new(
        deps.programs.clone(),
        validator,
        deps.store.clone(),
        deps.clock.clone(),
    )
}

fn program_with_cover(title: &str, cover_bytes: Vec<u8>) -> CreateProgram {
    CreateProgram {
        title: title.to_owned(),
        description: None,
        cover_bytes: Some(cover_bytes),
    }
}

#[tokio::test]
async fn covers_are_validated_and_capped() {
    let deps = TestDeps::new();
    let service = service_with(&deps, Arc::new(FakeCoverValidator));
    let garbage = b"definitely not an image".to_vec();
    let result = service
        .handle(program_with_cover("Bad cover", garbage))
        .await;
    assert_eq!(result.unwrap_err(), ApplicationError::UnsupportedMedia);
    assert!(deps.store.is_empty());

    let mut oversized = vec![0u8; MAX_COVER_BYTES + 12];
    oversized[0] = 0x89;
    oversized[1] = b'P';
    oversized[2] = b'N';
    oversized[3] = b'G';
    let result = service
        .handle(program_with_cover("Big cover", oversized))
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
        let deps = TestDeps::new();
        let service = service_with(&deps, Arc::new(FailingCoverValidator(error)));
        let result = service
            .handle(program_with_cover(
                "Cover mapping",
                b"RIFF....WEBP....".to_vec(),
            ))
            .await;
        assert_eq!(result.unwrap_err(), expected, "{error:?}");
        assert!(deps.store.is_empty());
        assert!(deps.programs.0.programs.lock().unwrap().is_empty());
    }
}
