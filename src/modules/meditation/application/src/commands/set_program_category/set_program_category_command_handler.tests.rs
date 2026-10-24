use crate::commands::set_program_category::{
    SetProgramCategoryCommand, SetProgramCategoryCommandHandler,
};
use crate::ports::CommandHandler;
use crate::ports::{CategoryRepository, ProgramRepository};
use crate::test_support::TestDeps;
use crate::ApplicationError;

fn service(deps: &TestDeps) -> SetProgramCategoryCommandHandler {
    SetProgramCategoryCommandHandler::new(deps.programs.clone(), deps.categories.clone())
}

async fn category_of(deps: &TestDeps, program_id: i64) -> Option<i64> {
    deps.programs
        .find_by_id(program_id)
        .await
        .unwrap()
        .unwrap()
        .category_id
}

#[tokio::test]
async fn assigns_an_existing_category() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    let category = deps
        .categories
        .insert("Sleep".into(), deps.clock.0)
        .await
        .unwrap();

    service(&deps)
        .handle(SetProgramCategoryCommand {
            program_id: program,
            category_id: Some(category.id),
        })
        .await
        .expect("assign");

    assert_eq!(category_of(&deps, program).await, Some(category.id));
}

#[tokio::test]
async fn none_clears_the_category() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    let category = deps
        .categories
        .insert("Sleep".into(), deps.clock.0)
        .await
        .unwrap();
    deps.programs
        .set_category(program, Some(category.id))
        .await
        .unwrap();

    service(&deps)
        .handle(SetProgramCategoryCommand {
            program_id: program,
            category_id: None,
        })
        .await
        .expect("clear");

    assert_eq!(category_of(&deps, program).await, None);
}

#[tokio::test]
async fn unknown_category_is_not_found_and_changes_nothing() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;

    let result = service(&deps)
        .handle(SetProgramCategoryCommand {
            program_id: program,
            category_id: Some(9999),
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
    assert_eq!(category_of(&deps, program).await, None);
}

#[tokio::test]
async fn unknown_program_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(SetProgramCategoryCommand {
            program_id: 9999,
            category_id: None,
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
