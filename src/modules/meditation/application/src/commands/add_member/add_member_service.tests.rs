use crate::commands::add_member::{AddMember, AddMemberHandler, AddMemberService};
use crate::ports::MembershipRepository;
use crate::test_support::TestDeps;
use crate::ApplicationError;

const UNKNOWN_ID: i64 = 9999;

fn service(deps: &TestDeps) -> AddMemberService {
    AddMemberService::new(
        deps.meditations.clone(),
        deps.programs.clone(),
        deps.memberships.clone(),
        deps.clock.clone(),
    )
}

#[tokio::test]
async fn adds_the_meditation_to_a_second_program() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let other_program = deps.seed_program("Other").await;

    service(&deps)
        .handle(AddMember {
            program_id: other_program,
            meditation_id: meditation.id(),
        })
        .await
        .expect("add member");

    let members = deps
        .memberships
        .list_by_program(other_program)
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].meditation_id, meditation.id());
}

#[tokio::test]
async fn adding_twice_keeps_a_single_membership() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let other_program = deps.seed_program("Other").await;
    let command = || AddMember {
        program_id: other_program,
        meditation_id: meditation.id(),
    };

    service(&deps).handle(command()).await.expect("first add");
    service(&deps).handle(command()).await.expect("second add");

    let members = deps
        .memberships
        .list_by_program(other_program)
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
}

#[tokio::test]
async fn unknown_meditation_is_not_found() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Program").await;

    let result = service(&deps)
        .handle(AddMember {
            program_id: program,
            meditation_id: UNKNOWN_ID,
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}

#[tokio::test]
async fn unknown_program_is_not_found() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    let result = service(&deps)
        .handle(AddMember {
            program_id: UNKNOWN_ID,
            meditation_id: meditation.id(),
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
