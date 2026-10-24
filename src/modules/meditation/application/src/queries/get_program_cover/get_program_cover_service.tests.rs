use crate::commands::replace_program_cover::{
    ReplaceProgramCover, ReplaceProgramCoverHandler, ReplaceProgramCoverService,
};
use crate::queries::get_program_cover::{
    GetProgramCover, GetProgramCoverHandler, GetProgramCoverService,
};
use crate::test_support::{user_a, TestDeps};
use crate::ApplicationError;

const PNG_MAGIC: [u8; 4] = [0x89, b'P', b'N', b'G'];
const JPEG_MAGIC: [u8; 3] = [0xFF, 0xD8, 0xFF];

fn service(deps: &TestDeps) -> GetProgramCoverService {
    GetProgramCoverService::new(deps.programs.clone(), deps.store.clone())
}

async fn store_cover(deps: &TestDeps, program_id: i64, bytes: &[u8]) {
    ReplaceProgramCoverService::new(deps.programs.clone(), deps.store.clone())
        .handle(ReplaceProgramCover {
            requester_id: user_a(),
            program_id,
            bytes: bytes.to_vec(),
        })
        .await
        .expect("store cover");
}

async fn content_type_for(bytes: &[u8]) -> &'static str {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    store_cover(&deps, program, bytes).await;
    service(&deps)
        .handle(GetProgramCover {
            requester_id: user_a(),
            program_id: program,
        })
        .await
        .unwrap()
        .expect("cover")
        .content_type
}

#[tokio::test]
async fn returns_stored_bytes_with_sniffed_content_type() {
    assert_eq!(content_type_for(&PNG_MAGIC).await, "image/png");
    assert_eq!(content_type_for(&JPEG_MAGIC).await, "image/jpeg");
    assert_eq!(content_type_for(b"RIFFxxxxWEBP").await, "image/webp");
}

#[tokio::test]
async fn returns_the_exact_bytes() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    store_cover(&deps, program, b"RIFFxxxxWEBP").await;

    let cover = service(&deps)
        .handle(GetProgramCover {
            requester_id: user_a(),
            program_id: program,
        })
        .await
        .unwrap()
        .expect("cover");

    assert_eq!(cover.bytes, b"RIFFxxxxWEBP");
}

#[tokio::test]
async fn program_without_cover_yields_none() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;

    let cover = service(&deps)
        .handle(GetProgramCover {
            requester_id: user_a(),
            program_id: program,
        })
        .await
        .unwrap();

    assert!(cover.is_none());
}

#[tokio::test]
async fn unknown_program_yields_none() {
    let deps = TestDeps::new();

    let cover = service(&deps)
        .handle(GetProgramCover {
            requester_id: user_a(),
            program_id: 9999,
        })
        .await
        .unwrap();

    assert!(cover.is_none());
}

#[tokio::test]
async fn store_failure_is_a_storage_error() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    store_cover(&deps, program, b"cover").await;
    *deps.store.failures.lock().unwrap() = true;

    let result = service(&deps)
        .handle(GetProgramCover {
            requester_id: user_a(),
            program_id: program,
        })
        .await;

    assert!(matches!(result, Err(ApplicationError::Storage(_))));
}
