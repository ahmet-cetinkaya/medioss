//! Persists the caller's playback position.
use crate::commands::ensure_meditation_exists;
use crate::ports::{MeditationRepository, PlaybackStateRepository};
use crate::ApplicationError;
use kernel::UserId;

/// Records the caller's position in media time (spec FR-015).
pub async fn record_position<R, P>(
    repository: &R,
    playback: &P,
    user_id: UserId,
    meditation_id: i64,
    position_ms: i64,
) -> Result<(), ApplicationError>
where
    R: MeditationRepository,
    P: PlaybackStateRepository,
{
    ensure_meditation_exists(repository, meditation_id).await?;
    if position_ms < 0 {
        return Err(ApplicationError::InvalidInput("position must be >= 0"));
    }
    playback
        .upsert_position(user_id, meditation_id, position_ms)
        .await
        .map_err(ApplicationError::Repository)
}
