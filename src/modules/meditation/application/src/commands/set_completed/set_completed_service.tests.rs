use chrono::Duration;

use crate::commands::record_completion::{
    RecordCompletion, RecordCompletionHandler, RecordCompletionService,
};
use crate::commands::set_completed::{SetCompleted, SetCompletedHandler, SetCompletedService};
use crate::ports::{PlaybackStateRepository, SessionRepository};
use crate::test_support::{user_a, TestDeps};

#[tokio::test]
async fn toggle_completed_never_touches_sessions_or_position() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    RecordCompletionService::new(
        deps.meditations.clone(),
        deps.playback.clone(),
        deps.sessions.clone(),
    )
    .handle(RecordCompletion {
        user_id: user_a(),
        meditation_id: meditation.id(),
        started_at: started,
        finished_at: started + Duration::seconds(30),
    })
    .await
    .expect("completion");

    SetCompletedService::new(deps.meditations.clone(), deps.playback.clone())
        .handle(SetCompleted {
            user_id: user_a(),
            meditation_id: meditation.id(),
            completed: false,
        })
        .await
        .expect("toggle off");
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
        .unwrap();
    assert!(!state.completed);
}
