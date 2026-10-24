use crate::ports::QueryHandler;
use crate::ports::{MembershipRepository, ProgramRepository};
use crate::queries::get_meditation_detail::{
    GetMeditationDetailQuery, GetMeditationDetailQueryHandler,
};
use crate::test_support::TestDeps;
use crate::ApplicationError;

fn service(deps: &TestDeps) -> GetMeditationDetailQueryHandler {
    GetMeditationDetailQueryHandler::new(
        deps.meditations.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
    )
}

#[tokio::test]
async fn includes_the_home_program() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Home").await;
    let meditation = deps.seed_meditation_in(program, "Sit").await;

    let detail = service(&deps)
        .handle(GetMeditationDetailQuery {
            meditation_id: meditation.id(),
        })
        .await
        .expect("detail");

    assert_eq!(detail.meditation.id(), meditation.id());
    assert_eq!(detail.program.expect("program").title, "Home");
}

#[tokio::test]
async fn meditation_without_membership_has_no_program() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Home").await;
    let meditation = deps.seed_meditation_in(program, "Sit").await;
    deps.memberships
        .remove(program, meditation.id())
        .await
        .unwrap();

    let detail = service(&deps)
        .handle(GetMeditationDetailQuery {
            meditation_id: meditation.id(),
        })
        .await
        .expect("detail");

    assert!(detail.program.is_none());
}

#[tokio::test]
async fn missing_program_row_degrades_to_no_program() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Home").await;
    let meditation = deps.seed_meditation_in(program, "Sit").await;
    deps.programs.delete(program).await.unwrap();

    let detail = service(&deps)
        .handle(GetMeditationDetailQuery {
            meditation_id: meditation.id(),
        })
        .await
        .expect("detail");

    assert!(detail.program.is_none());
}

#[tokio::test]
async fn unknown_meditation_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(GetMeditationDetailQuery {
            meditation_id: 9999,
        })
        .await;

    assert_eq!(result.err(), Some(ApplicationError::NotFound));
}
