use crate::commands::remove_program_cover::{
    RemoveProgramCoverCommand, RemoveProgramCoverCommandHandler,
};
use crate::commands::replace_program_cover::{
    ReplaceProgramCoverCommand, ReplaceProgramCoverCommandHandler,
};
use crate::ports::{CommandHandler, ProgramRepository};
use crate::test_support::{user_a, TestDeps};
use crate::ApplicationError;

fn remove(deps: &TestDeps) -> RemoveProgramCoverCommandHandler {
    RemoveProgramCoverCommandHandler::new(deps.programs.clone(), deps.store.clone())
}

async fn give_cover(deps: &TestDeps, program_id: i64) -> String {
    ReplaceProgramCoverCommandHandler::new(deps.programs.clone(), deps.store.clone())
        .handle(ReplaceProgramCoverCommand {
            requester_id: user_a(),
            program_id,
            bytes: b"cover".to_vec(),
        })
        .await
        .unwrap();
    cover_key(deps, program_id).await.expect("cover key set")
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
async fn removing_clears_the_key_and_deletes_the_blob() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    let key = give_cover(&deps, program).await;

    remove(&deps)
        .handle(RemoveProgramCoverCommand {
            program_id: program,
        })
        .await
        .unwrap();

    assert_eq!(cover_key(&deps, program).await, None);
    assert!(!deps.store.contains_key(&key));
}

#[tokio::test]
async fn removing_a_missing_cover_succeeds_and_changes_nothing() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;

    remove(&deps)
        .handle(RemoveProgramCoverCommand {
            program_id: program,
        })
        .await
        .unwrap();

    assert_eq!(cover_key(&deps, program).await, None);
}

#[tokio::test]
async fn an_unknown_program_is_not_found() {
    let deps = TestDeps::new();

    let result = remove(&deps)
        .handle(RemoveProgramCoverCommand { program_id: 999 })
        .await;

    assert!(matches!(result, Err(ApplicationError::NotFound)));
}
