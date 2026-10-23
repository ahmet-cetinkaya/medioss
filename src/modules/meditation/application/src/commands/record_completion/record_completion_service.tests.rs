use chrono::Duration;

use crate::commands::record_completion::{
    RecordCompletion, RecordCompletionHandler, RecordCompletionService,
};
use crate::commands::record_position::{
    RecordPosition, RecordPositionHandler, RecordPositionService,
};
use crate::ports::{PlaybackStateRepository, SessionRepository};
use crate::test_support::{user_a, user_b, TestDeps};

fn service(deps: &TestDeps) -> RecordCompletionService {
    RecordCompletionService::new(
        deps.meditations.clone(),
        deps.playback.clone(),
        deps.sessions.clone(),
    )
}

#[tokio::test]
async fn completion_is_idempotent_sets_completed_and_resets_position() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    let finished = started + Duration::seconds(60);

    RecordPositionService::new(deps.meditations.clone(), deps.playback.clone())
        .handle(RecordPosition {
            user_id: user_a(),
            meditation_id: meditation.id(),
            position_ms: 30_000,
        })
        .await
        .expect("position");

    let completion = RecordCompletion {
        user_id: user_a(),
        meditation_id: meditation.id(),
        started_at: started,
        finished_at: finished,
    };
    let first = service(&deps)
        .handle(completion.clone())
        .await
        .expect("completion");
    let retry = service(&deps).handle(completion).await.expect("retry");

    assert_eq!(first.id, retry.id);
    assert_eq!(
        deps.sessions
            .count_for(user_a(), meditation.id())
            .await
            .unwrap(),
        1
    );
    let state = deps
        .playback
        .get(user_a(), meditation.id())
        .await
        .unwrap()
        .expect("state");
    assert_eq!(state.position_ms, 0);
    assert!(state.completed);
}

#[tokio::test]
async fn completion_state_is_per_user() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    service(&deps)
        .handle(RecordCompletion {
            user_id: user_a(),
            meditation_id: meditation.id(),
            started_at: started,
            finished_at: started + Duration::seconds(30),
        })
        .await
        .expect("a completes");

    let b_state = deps.playback.get(user_b(), meditation.id()).await.unwrap();
    assert!(b_state.is_none());
}
