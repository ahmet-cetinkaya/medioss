//! Flips the caller's completed flag only.
use crate::commands::ensure_meditation_exists;
use crate::ports::{MeditationRepository, PlaybackStateRepository};
use crate::ApplicationError;
use kernel::UserId;

/// Toggles the caller's completed state; never touches sessions or the
/// stored position (spec FR-022).
pub async fn set_completed<R, P>(
    repository: &R,
    playback: &P,
    user_id: UserId,
    meditation_id: i64,
    completed: bool,
) -> Result<(), ApplicationError>
where
    R: MeditationRepository,
    P: PlaybackStateRepository,
{
    ensure_meditation_exists(repository, meditation_id).await?;
    playback
        .set_completed(user_id, meditation_id, completed)
        .await
        .map_err(ApplicationError::Repository)
}
