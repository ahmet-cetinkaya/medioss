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
    R: MeditationRepository + ?Sized,
    P: PlaybackStateRepository + ?Sized,
{
    ensure_meditation_exists(repository, meditation_id).await?;
    // Bounded well above any real meditation's duration; rejects pathological
    // inputs before they can reach the progress-percent computation instead
    // of relying solely on wide-integer arithmetic there.
    const MAX_POSITION_MS: i64 = 30 * 24 * 60 * 60 * 1000;
    if !(0..=MAX_POSITION_MS).contains(&position_ms) {
        return Err(ApplicationError::InvalidInput(
            "position must be between 0 and 30 days in milliseconds",
        ));
    }
    playback
        .upsert_position(user_id, meditation_id, position_ms)
        .await
        .map_err(ApplicationError::Repository)
}
