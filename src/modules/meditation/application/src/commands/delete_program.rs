//! Deletes a program: memberships go, the folder goes, and member
//! meditations survive by relocating their files to the shared `_unfiled`
//! folder (spec FR-028 + user direction).
use crate::ports::{BlobStore, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

const UNFILED: &str = "_unfiled";

/// Deletes a program (spec FR-028). Every meditation whose file actually
/// lives in the program's folder is relocated to `_unfiled/` first (decided
/// by `storage_key` prefix, not by the membership snapshot, so a concurrent
/// upload or a missing membership row can never leave an orphaned or
/// destroyed blob); only once every file is safely out of the folder are
/// the memberships and the program row removed and the folder deleted.
///
/// Idempotent and safe to retry: a meditation whose key no longer starts
/// with the program's folder prefix is treated as already relocated, so a
/// retry after a partial failure only finishes the remaining work.
pub async fn delete_program<P, M, S, R>(
    programs: &P,
    memberships: &M,
    meditations: &R,
    store: &S,
    program_id: i64,
) -> Result<(), ApplicationError>
where
    P: ProgramRepository + ?Sized,
    M: MembershipRepository + ?Sized,
    R: MeditationRepository + ?Sized,
    S: BlobStore + ?Sized,
{
    let Some(program) = programs
        .find_by_id(program_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Err(ApplicationError::NotFound);
    };
    let prefix = format!("{}/", program.folder);
    for meditation in meditations
        .list_by_storage_prefix(&prefix)
        .await
        .map_err(ApplicationError::Repository)?
    {
        let new_key = store
            .relocate(meditation.storage_key(), UNFILED, meditation.name(), "opus")
            .await
            .map_err(ApplicationError::Storage)?;
        meditations
            .set_storage_key(meditation.id(), &new_key)
            .await
            .map_err(ApplicationError::Repository)?;
    }
    for membership in memberships
        .list_by_program(program_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        memberships
            .remove(program_id, membership.meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
    }
    if programs
        .delete(program_id)
        .await
        .map_err(ApplicationError::Repository)?
        .is_none()
    {
        // Deleted concurrently between the lookup above and here: the
        // relocation and membership cleanup already ran, so this is not an
        // error — report success rather than a confusing NotFound.
        return Ok(());
    }
    store
        .delete_folder(&program.folder)
        .await
        .map_err(ApplicationError::Storage)?;
    Ok(())
}
