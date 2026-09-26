//! Updates the author's note.
use crate::ports::{Clock, NoteRepository};
use crate::ApplicationError;
use kernel::UserId;

/// Updates the author's note; another author's or unknown id is NotFound
/// (spec FR-036/037).
pub async fn update_note<N, K>(
    notes: &N,
    clock: &K,
    note_id: i64,
    user_id: UserId,
    text: &str,
) -> Result<(), ApplicationError>
where
    N: NoteRepository,
    K: Clock,
{
    let text = text.trim();
    if text.is_empty() {
        return Err(ApplicationError::InvalidInput(
            "note text must not be empty",
        ));
    }
    let updated = notes
        .update(note_id, user_id, text.to_owned(), clock.now().await)
        .await
        .map_err(ApplicationError::Repository)?;
    if updated {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}
