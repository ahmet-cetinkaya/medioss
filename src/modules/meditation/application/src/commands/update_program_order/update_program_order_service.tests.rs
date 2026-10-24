use crate::commands::update_program_order::{
    UpdateProgramOrder, UpdateProgramOrderHandler, UpdateProgramOrderService,
};
use crate::ports::MembershipRepository;
use crate::test_support::TestDeps;
use crate::ApplicationError;

fn service(deps: &TestDeps) -> UpdateProgramOrderService {
    UpdateProgramOrderService::new(deps.programs.clone(), deps.memberships.clone())
}

async fn member_ids(deps: &TestDeps, program_id: i64) -> Vec<i64> {
    deps.memberships
        .list_by_program(program_id)
        .await
        .unwrap()
        .into_iter()
        .map(|member| member.meditation_id)
        .collect()
}

async fn program_with_two(deps: &TestDeps) -> (i64, i64, i64) {
    let program = deps.seed_program("Program").await;
    let first = deps.seed_meditation_in(program, "First").await.id();
    let second = deps.seed_meditation_in(program, "Second").await.id();
    (program, first, second)
}

#[tokio::test]
async fn reorders_members() {
    let deps = TestDeps::new();
    let (program, first, second) = program_with_two(&deps).await;
    assert_eq!(member_ids(&deps, program).await, vec![first, second]);

    service(&deps)
        .handle(UpdateProgramOrder {
            program_id: program,
            meditation_ids: vec![second, first],
        })
        .await
        .expect("reorder");

    assert_eq!(member_ids(&deps, program).await, vec![second, first]);
}

#[tokio::test]
async fn missing_member_is_an_invalid_order() {
    let deps = TestDeps::new();
    let (program, first, _) = program_with_two(&deps).await;

    let result = service(&deps)
        .handle(UpdateProgramOrder {
            program_id: program,
            meditation_ids: vec![first],
        })
        .await;

    assert_eq!(result, Err(ApplicationError::InvalidProgramOrder));
}

#[tokio::test]
async fn foreign_or_duplicate_ids_are_an_invalid_order() {
    let deps = TestDeps::new();
    let (program, first, second) = program_with_two(&deps).await;

    let foreign = service(&deps)
        .handle(UpdateProgramOrder {
            program_id: program,
            meditation_ids: vec![first, 9999],
        })
        .await;
    let duplicate = service(&deps)
        .handle(UpdateProgramOrder {
            program_id: program,
            meditation_ids: vec![first, first],
        })
        .await;

    assert_eq!(foreign, Err(ApplicationError::InvalidProgramOrder));
    assert_eq!(duplicate, Err(ApplicationError::InvalidProgramOrder));
    assert_eq!(member_ids(&deps, program).await, vec![first, second]);
}

#[tokio::test]
async fn empty_program_accepts_an_empty_order() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Empty").await;

    let result = service(&deps)
        .handle(UpdateProgramOrder {
            program_id: program,
            meditation_ids: vec![],
        })
        .await;

    assert_eq!(result, Ok(()));
}

#[tokio::test]
async fn unknown_program_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(UpdateProgramOrder {
            program_id: 9999,
            meditation_ids: vec![],
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
