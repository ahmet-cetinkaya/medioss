use crate::ports::PlaybackStateRepository;
use crate::ports::QueryHandler;
use crate::queries::list_recent_programs::{
    ListRecentProgramsQuery, ListRecentProgramsQueryHandler,
};
use crate::test_support::{user_a, TestDeps};

#[tokio::test]
async fn recent_programs_dedupes_by_program_most_recent_first_and_honors_limit() {
    let deps = TestDeps::new();
    let program_a = deps.seed_program("A").await;
    let program_b = deps.seed_program("B").await;
    let med_a1 = deps.seed_meditation_in(program_a, "A1").await;
    let med_a2 = deps.seed_meditation_in(program_a, "A2").await;
    let med_b = deps.seed_meditation_in(program_b, "B1").await;

    // Touch order (oldest to newest): A1, B1, A2. A2's touch should keep A
    // as the most-recent program (A2 is newer than B1's touch of B), and
    // A1's earlier touch of the same program must not appear twice.
    deps.playback
        .upsert_position(user_a(), med_a1.id(), 100)
        .await
        .expect("touch a1");
    deps.playback
        .upsert_position(user_a(), med_b.id(), 100)
        .await
        .expect("touch b1");
    deps.playback
        .upsert_position(user_a(), med_a2.id(), 100)
        .await
        .expect("touch a2");

    let service = ListRecentProgramsQueryHandler::new(
        deps.playback.clone(),
        deps.memberships.clone(),
        deps.programs.clone(),
    );
    let all = service
        .handle(ListRecentProgramsQuery {
            user_id: user_a(),
            limit: None,
        })
        .await
        .expect("query");
    assert_eq!(all.len(), 2, "each program appears once");
    assert_eq!(
        all[0].program.id, program_a,
        "most recently touched program first"
    );
    assert_eq!(all[1].program.id, program_b);

    let limited = service
        .handle(ListRecentProgramsQuery {
            user_id: user_a(),
            limit: Some(1),
        })
        .await
        .expect("query");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].program.id, program_a);
}
