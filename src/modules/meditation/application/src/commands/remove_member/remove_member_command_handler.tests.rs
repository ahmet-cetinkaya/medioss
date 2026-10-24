use crate::commands::remove_member::{RemoveMemberCommand, RemoveMemberCommandHandler};
use crate::ports::CommandHandler;
use crate::ports::MembershipRepository;
use crate::test_support::TestDeps;
use crate::ApplicationError;

fn service(deps: &TestDeps) -> RemoveMemberCommandHandler {
    RemoveMemberCommandHandler::new(deps.programs.clone(), deps.memberships.clone())
}

#[tokio::test]
async fn removes_the_membership() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;
    let meditation = deps.seed_meditation_in(program, "Sit").await;

    service(&deps)
        .handle(RemoveMemberCommand {
            program_id: program,
            meditation_id: meditation.id(),
        })
        .await
        .expect("remove");

    assert!(deps
        .memberships
        .list_by_program(program)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn unknown_pair_in_a_known_program_succeeds() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;

    let result = service(&deps)
        .handle(RemoveMemberCommand {
            program_id: program,
            meditation_id: 9999,
        })
        .await;

    assert_eq!(result, Ok(()));
}

#[tokio::test]
async fn unknown_program_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(RemoveMemberCommand {
            program_id: 9999,
            meditation_id: 1,
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
