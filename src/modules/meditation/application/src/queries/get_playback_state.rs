//! The caller's playback state for one meditation.
use crate::ports::{MeditationRepository, PlaybackState, PlaybackStateRepository};
use crate::ApplicationError;
use kernel::UserId;

/// Returns the caller's state; zero/false when never played (spec FR-016).
pub async fn get_playback_state<R, P>(
    meditations: &R,
    playback: &P,
    user_id: UserId,
    meditation_id: i64,
) -> Result<PlaybackState, ApplicationError>
where
    R: MeditationRepository + ?Sized,
    P: PlaybackStateRepository + ?Sized,
{
    crate::queries::get_meditation(meditations, meditation_id).await?;
    Ok(playback
        .get(user_id, meditation_id)
        .await
        .map_err(ApplicationError::Repository)?
        .unwrap_or(PlaybackState {
            position_ms: 0,
            completed: false,
        }))
}
