//! Deletes the author's note.
use crate::ports::NoteRepository;
use crate::ApplicationError;
use kernel::UserId;

/// Deletes the author's note on `meditation_id` (spec FR-036/037).
pub async fn delete_note<N>(
    notes: &N,
    note_id: i64,
    meditation_id: i64,
    user_id: UserId,
) -> Result<(), ApplicationError>
where
    N: NoteRepository + ?Sized,
{
    let deleted = notes
        .delete(note_id, meditation_id, user_id)
        .await
        .map_err(ApplicationError::Repository)?;
    if deleted {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}
