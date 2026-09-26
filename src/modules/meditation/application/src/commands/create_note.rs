//! Creates a practice note on a meditation for its author.
use crate::commands::ensure_meditation_exists;
use crate::ports::{Clock, MeditationRepository, Note, NoteRepository};
use crate::ApplicationError;
use kernel::UserId;

/// Creates a note (spec FR-034).
pub async fn create_note<R, N, K>(
    meditations: &R,
    notes: &N,
    clock: &K,
    user_id: UserId,
    meditation_id: i64,
    text: &str,
) -> Result<Note, ApplicationError>
where
    R: MeditationRepository,
    N: NoteRepository,
    K: Clock,
{
    ensure_meditation_exists(meditations, meditation_id).await?;
    let text = text.trim();
    if text.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "note text must not be empty",
        ));
    }
    let now = clock.now().await;
    notes
        .insert(user_id, meditation_id, text.to_owned(), now)
        .await
        .map_err(ApplicationError::Repository)
}
