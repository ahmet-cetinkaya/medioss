use crate::ports::QueryHandler;
use crate::queries::list_program_members::{
    ListProgramMembersQuery, ListProgramMembersQueryHandler,
};
use crate::test_support::TestDeps;
use crate::ApplicationError;

fn service(deps: &TestDeps) -> ListProgramMembersQueryHandler {
    ListProgramMembersQueryHandler::new(deps.programs.clone(), deps.memberships.clone())
}

#[tokio::test]
async fn lists_members_in_position_order() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    let first = deps.seed_meditation_in(program, "First").await.id();
    let second = deps.seed_meditation_in(program, "Second").await.id();

    let members = service(&deps)
        .handle(ListProgramMembersQuery {
            program_id: program,
        })
        .await
        .expect("members");

    let ids: Vec<i64> = members.iter().map(|member| member.meditation_id).collect();
    assert_eq!(ids, vec![first, second]);
}

#[tokio::test]
async fn known_empty_program_lists_nothing() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Empty").await;

    let members = service(&deps)
        .handle(ListProgramMembersQuery {
            program_id: program,
        })
        .await
        .expect("members");

    assert!(members.is_empty());
}

#[tokio::test]
async fn unknown_program_is_not_found_rather_than_empty() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(ListProgramMembersQuery { program_id: 9999 })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
