//! Records one completed playback pass, idempotently.
use chrono::{DateTime, Utc};

use crate::commands::ensure_meditation_exists;
use crate::ports::{
    MeditationRepository, MeditationSession, PlaybackStateRepository, SessionRepository,
};
use crate::ApplicationError;
use kernel::UserId;

/// Records a completion. Idempotent per (user, meditation, started_at): a
/// retry returns the same session, flips `completed` to true, and resets
/// the position (spec FR-019..021).
pub async fn record_completion<R, P, E>(
    repository: &R,
    playback: &P,
    sessions: &E,
    user_id: UserId,
    meditation_id: i64,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
) -> Result<MeditationSession, ApplicationError>
where
    R: MeditationRepository,
    P: PlaybackStateRepository,
    E: SessionRepository,
{
    ensure_meditation_exists(repository, meditation_id).await?;
    if finished_at < started_at {
        return Err(ApplicationError::InvalidInput(
            "finished_at must not precede started_at",
        ));
    }
    let session = sessions
        .record_if_new(user_id, meditation_id, started_at, finished_at)
        .await
        .map_err(ApplicationError::Repository)?;
    playback
        .set_completed(user_id, meditation_id, true)
        .await
        .map_err(ApplicationError::Repository)?;
    playback
        .reset_position(user_id, meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    Ok(session)
}
