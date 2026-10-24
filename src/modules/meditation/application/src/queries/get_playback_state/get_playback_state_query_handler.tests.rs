use crate::ports::PlaybackState;
use crate::ports::QueryHandler;
use crate::queries::get_playback_state::{GetPlaybackStateQuery, GetPlaybackStateQueryHandler};
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

fn service(deps: &TestDeps) -> GetPlaybackStateQueryHandler {
    GetPlaybackStateQueryHandler::new(deps.meditations.clone(), deps.playback.clone())
}

#[tokio::test]
async fn never_played_is_zero_and_not_completed() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;

    let state = service(&deps)
        .handle(GetPlaybackStateQuery {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .expect("state");

    assert_eq!(
        state,
        PlaybackState {
            position_ms: 0,
            completed: false
        }
    );
}

#[tokio::test]
async fn returns_the_stored_state_for_that_user_only() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let stored = PlaybackState {
        position_ms: 42_000,
        completed: true,
    };
    deps.playback
        .0
        .seed_playback(user_a(), meditation.id(), stored);

    let own = service(&deps)
        .handle(GetPlaybackStateQuery {
            user_id: user_a(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();
    let other = service(&deps)
        .handle(GetPlaybackStateQuery {
            user_id: user_b(),
            meditation_id: meditation.id(),
        })
        .await
        .unwrap();

    assert_eq!(own, stored);
    assert_eq!(other.position_ms, 0);
}

#[tokio::test]
async fn unknown_meditation_is_not_found() {
    let deps = TestDeps::new();

    let result = service(&deps)
        .handle(GetPlaybackStateQuery {
            user_id: user_a(),
            meditation_id: 9999,
        })
        .await;

    assert_eq!(result, Err(ApplicationError::NotFound));
}
