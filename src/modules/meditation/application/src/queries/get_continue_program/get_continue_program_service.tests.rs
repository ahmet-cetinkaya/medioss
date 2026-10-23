use crate::ports::PlaybackStateRepository;
use crate::queries::get_continue_program::{
    GetContinueProgram, GetContinueProgramHandler, GetContinueProgramService,
};
use crate::test_support::{user_a, TestDeps};

fn service(deps: &TestDeps) -> GetContinueProgramService {
    GetContinueProgramService::new(
        deps.playback.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
    )
}

#[tokio::test]
async fn continue_program_resolves_the_home_program_of_the_last_touch() {
    let deps = TestDeps::new();
    let program = deps.seed_program("Morning").await;
    let meditation = deps.seed_meditation_in(program, "M1").await;
    deps.playback
        .upsert_position(user_a(), meditation.id(), 1_000)
        .await
        .expect("touch");

    let found = service(&deps)
        .handle(GetContinueProgram { user_id: user_a() })
        .await
        .expect("query")
        .expect("some");
    assert_eq!(found.program.id, program);
    assert_eq!(found.meditation_id, meditation.id());
}

#[tokio::test]
async fn continue_program_is_none_for_an_untouched_user() {
    let deps = TestDeps::new();
    let found = service(&deps)
        .handle(GetContinueProgram { user_id: user_a() })
        .await
        .expect("query");
    assert!(found.is_none());
}

#[tokio::test]
async fn continue_program_falls_back_past_a_deleted_meditations_touch() {
    // The most recently touched meditation has no membership at all (as if
    // its program had been deleted and the meditation later removed too,
    // or the membership row otherwise vanished); the query must not give up
    // and must resolve to the next most-recently-touched live one instead.
    let deps = TestDeps::new();
    let program = deps.seed_program("Evening").await;
    let live = deps.seed_meditation_in(program, "Live").await;
    deps.playback
        .upsert_position(user_a(), live.id(), 500)
        .await
        .expect("touch live");
    // A touch on a meditation id with no membership row anywhere.
    let orphan_id = live.id() + 999;
    deps.playback
        .upsert_position(user_a(), orphan_id, 900)
        .await
        .expect("touch orphan");

    let found = service(&deps)
        .handle(GetContinueProgram { user_id: user_a() })
        .await
        .expect("query")
        .expect("some");
    assert_eq!(found.meditation_id, live.id());
}
