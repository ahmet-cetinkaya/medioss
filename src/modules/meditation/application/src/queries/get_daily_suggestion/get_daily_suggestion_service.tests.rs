use crate::queries::get_daily_suggestion::{
    GetDailySuggestion, GetDailySuggestionHandler, GetDailySuggestionService,
};
use crate::test_support::TestDeps;

fn service(deps: &TestDeps) -> GetDailySuggestionService {
    GetDailySuggestionService::new(
        deps.meditations.clone(),
        deps.clock.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
    )
}

#[tokio::test]
async fn daily_suggestion_is_the_same_pick_for_repeated_calls_same_day() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Library Home").await;
    deps.seed_meditation_in(program, "A").await;
    deps.seed_meditation_in(program, "B").await;
    deps.seed_meditation_in(program, "C").await;

    let first = service(&deps)
        .handle(GetDailySuggestion)
        .await
        .expect("query")
        .expect("some");
    let second = service(&deps)
        .handle(GetDailySuggestion)
        .await
        .expect("query")
        .expect("some");
    assert_eq!(first.meditation.id(), second.meditation.id());
    assert!(first.program.is_some());
}

#[tokio::test]
async fn daily_suggestion_is_none_for_an_empty_library() {
    let deps = TestDeps::new();
    let found = service(&deps)
        .handle(GetDailySuggestion)
        .await
        .expect("query");
    assert!(found.is_none());
}
