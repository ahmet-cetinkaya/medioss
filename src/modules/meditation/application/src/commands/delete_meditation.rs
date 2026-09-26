//! Deletes a meditation with its bytes, memberships, and every user's notes.
use crate::ports::{BlobStore, MeditationRepository, MembershipRepository, NoteRepository};
use crate::ApplicationError;

/// Deletes a meditation entirely (spec FR-014). Row removal is the source
/// of truth: stored bytes are removed after the row is gone, and a store
/// failure surfaces without resurrecting the row.
pub async fn delete_meditation<R, M, N, S>(
    repository: &R,
    memberships: &M,
    notes: &N,
    store: &S,
    meditation_id: i64,
) -> Result<(), ApplicationError>
where
    R: MeditationRepository,
    M: MembershipRepository,
    N: NoteRepository,
    S: BlobStore,
{
    let Some(meditation) = repository
        .find_by_id(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Err(ApplicationError::NotFound);
    };
    memberships
        .remove_for_meditation(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    notes
        .delete_for_meditation(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    repository
        .delete(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    store
        .delete(meditation.storage_key())
        .await
        .map_err(ApplicationError::Storage)?;
    Ok(())
}
