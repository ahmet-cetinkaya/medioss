use crate::commands::update_program::{UpdateProgram, UpdateProgramHandler, UpdateProgramService};
use crate::ports::ProgramRepository;
use crate::services::MAX_NAME_LENGTH;
use crate::test_support::TestDeps;
use crate::ApplicationError;

fn service(deps: &TestDeps) -> UpdateProgramService {
    UpdateProgramService::new(deps.programs.clone())
}

fn command(program_id: i64, title: &str, description: Option<&str>) -> UpdateProgram {
    UpdateProgram {
        program_id,
        title: title.into(),
        description: description.map(str::to_owned),
    }
}

#[tokio::test]
async fn updates_title_and_description_trimmed() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Old").await;

    service(&deps)
        .handle(command(program, "  New title  ", Some("  about  ")))
        .await
        .expect("update");

    let stored = deps.programs.find_by_id(program).await.unwrap().unwrap();
    assert_eq!(stored.title, "New title");
    assert_eq!(stored.description.as_deref(), Some("about"));
}

#[tokio::test]
async fn blank_description_is_cleared() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Old").await;
    service(&deps)
        .handle(command(program, "Old", Some("text")))
        .await
        .unwrap();

    service(&deps)
        .handle(command(program, "Old", Some("   ")))
        .await
        .unwrap();

    let stored = deps.programs.find_by_id(program).await.unwrap().unwrap();
    assert_eq!(stored.description, None);
}

#[tokio::test]
async fn invalid_title_is_rejected_and_keeps_the_old_one() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Old").await;
    let too_long = "x".repeat(MAX_NAME_LENGTH + 1);

    let blank = service(&deps).handle(command(program, "  ", None)).await;
    let long = service(&deps)
        .handle(command(program, &too_long, None))
        .await;

    assert!(matches!(blank, Err(ApplicationError::InvalidInput(_))));
    assert!(matches!(long, Err(ApplicationError::InvalidInput(_))));
    let stored = deps.programs.find_by_id(program).await.unwrap().unwrap();
    assert_eq!(stored.title, "Old");
}

#[tokio::test]
async fn unknown_program_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps).handle(command(9999, "Title", None)).await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
