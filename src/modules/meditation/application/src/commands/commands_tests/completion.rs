use chrono::Duration;

use super::super::*;
use super::helpers::*;
use crate::ports::{PlaybackStateRepository, SessionRepository};

#[tokio::test]
async fn completion_is_idempotent_sets_completed_and_resets_position() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    let finished = started + Duration::seconds(60);

    record_position(
        &deps.meditations,
        &deps.playback,
        user_a(),
        meditation.id(),
        30_000,
    )
    .await
    .expect("position");

    let first = record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        finished,
    )
    .await
    .expect("completion");
    let retry = record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        finished,
    )
    .await
    .expect("retry");

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
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        started + Duration::seconds(30),
    )
    .await
    .expect("a completes");

    let b_state = deps.playback.get(user_b(), meditation.id()).await.unwrap();
    assert!(b_state.is_none());
}

#[tokio::test]
async fn toggle_completed_never_touches_sessions_or_position() {
    let deps = Deps::new();
    let meditation = deps.seeded_meditation().await;
    let started = deps.clock.0;
    record_completion(
        &deps.meditations,
        &deps.playback,
        &deps.sessions,
        user_a(),
        meditation.id(),
        started,
        started + Duration::seconds(30),
    )
    .await
    .expect("completion");

    set_completed(
        &deps.meditations,
        &deps.playback,
        user_a(),
        meditation.id(),
        false,
    )
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
