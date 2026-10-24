use crate::commands::replace_program_cover::{
    ReplaceProgramCover, ReplaceProgramCoverHandler, ReplaceProgramCoverService,
};
use crate::ports::ProgramRepository;
use crate::test_support::{user_a, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> ReplaceProgramCoverService {
    ReplaceProgramCoverService::new(deps.programs.clone(), deps.store.clone())
}

fn command(program_id: i64, bytes: &[u8]) -> ReplaceProgramCover {
    ReplaceProgramCover {
        requester_id: user_a(),
        program_id,
        bytes: bytes.to_vec(),
    }
}

async fn cover_key(deps: &TestDeps, program_id: i64) -> Option<String> {
    deps.programs
        .find_by_id(program_id)
        .await
        .unwrap()
        .unwrap()
        .cover_storage_key
}

#[tokio::test]
async fn stores_the_cover_and_records_its_key() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;

    service(&deps)
        .handle(command(program, b"cover-one"))
        .await
        .expect("replace");

    let key = cover_key(&deps, program).await.expect("cover key set");
    assert!(deps.store.contains_key(&key));
}

#[tokio::test]
async fn replacing_deletes_the_previous_cover_blob() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    service(&deps)
        .handle(command(program, b"cover-one"))
        .await
        .unwrap();
    let old_key = cover_key(&deps, program).await.unwrap();

    service(&deps)
        .handle(command(program, b"cover-two"))
        .await
        .unwrap();

    let new_key = cover_key(&deps, program).await.unwrap();
    assert_ne!(old_key, new_key);
    assert!(!deps.store.contains_key(&old_key));
    assert!(deps.store.contains_key(&new_key));
    assert_eq!(deps.store.len(), 1);
}

#[tokio::test]
async fn unknown_program_is_not_found_and_stores_nothing() {
    let deps = TestDeps::new();

    let result = service(&deps).handle(command(9999, b"cover")).await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    assert!(deps.store.is_empty());
}

#[tokio::test]
async fn store_failure_is_a_storage_error_and_keeps_the_old_cover() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    service(&deps)
        .handle(command(program, b"cover-one"))
        .await
        .unwrap();
    let old_key = cover_key(&deps, program).await.unwrap();
    *deps.store.failures.lock().unwrap() = true;

    let result = service(&deps).handle(command(program, b"cover-two")).await;

    assert_eq!(
        result,
        Err(ApplicationError::Storage("store down".to_owned()))
    );
    assert_eq!(cover_key(&deps, program).await, Some(old_key));
}
