//! Lists the author's notes on one meditation.
use crate::ports::{MeditationRepository, Note, NoteRepository};
use crate::ApplicationError;
use kernel::UserId;

/// Lists the author's notes, newest first (spec FR-035).
pub async fn list_notes<R, N>(
    meditations: &R,
    notes: &N,
    user_id: UserId,
    meditation_id: i64,
) -> Result<Vec<Note>, ApplicationError>
where
    R: MeditationRepository,
    N: NoteRepository,
{
    crate::queries::get_meditation(meditations, meditation_id).await?;
    notes
        .list_for(user_id, meditation_id)
        .await
        .map_err(ApplicationError::Repository)
}
