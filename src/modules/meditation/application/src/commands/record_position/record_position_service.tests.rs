use crate::commands::record_position::{
    RecordPosition, RecordPositionHandler, RecordPositionService,
};
use crate::queries::get_playback_state::{
    GetPlaybackState, GetPlaybackStateHandler, GetPlaybackStateService,
};
use crate::test_support::{user_a, user_b, TestDeps};
use crate::ApplicationError;

#[tokio::test]
async fn position_reports_are_validated_and_default_to_zero() {
    let deps = TestDeps::new();
    let meditation = deps.seeded_meditation().await;
    let service = RecordPositionService::new(deps.meditations.clone(), deps.playback.clone());
    let unknown = service
        .handle(RecordPosition {
            user_id: user_a(),
            meditation_id: 999,
            position_ms: 1_000,
        })
        .await;
    assert_eq!(unknown.unwrap_err(), ApplicationError::NotFound);

    let negative = service
        .handle(RecordPosition {
            user_id: user_a(),
            meditation_id: meditation.id(),
            position_ms: -1,
        })
        .await;
    assert!(matches!(negative, Err(ApplicationError::InvalidInput(_))));

    let fresh = GetPlaybackStateService::new(deps.meditations.clone(), deps.playback.clone())
        .handle(GetPlaybackState {
            user_id: user_b(),
            meditation_id: meditation.id(),
        })
        .await
        .expect("state");
    assert_eq!(fresh.position_ms, 0);
    assert!(!fresh.completed);
}
