//! Deletes a program: memberships go, the folder goes, and member
//! meditations survive by relocating their files to the shared `_unfiled`
//! folder (spec FR-028 + user direction).
use crate::ports::{BlobStore, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

const UNFILED: &str = "_unfiled";

/// Deletes a program (spec FR-028). Every member meditation's file moves to
/// `_unfiled/` and its storage key is updated; the program folder and cover
/// are then removed.
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
        .delete(program_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Err(ApplicationError::NotFound);
    };
    for membership in memberships
        .list_by_program(program_id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        memberships
            .remove(program_id, membership.meditation_id)
            .await
            .map_err(ApplicationError::Repository)?;
        // Relocate the file only when it still lives under this program's
        // folder (an upload from a different program keeps its own folder).
        if let Some(meditation) = meditations
            .find_by_id(membership.meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            let key = meditation.storage_key().to_owned();
            if key.starts_with(&format!("{}/", program.folder)) {
                let name = meditation.name().to_owned();
                let new_key = store
                    .relocate(&key, UNFILED, &name, "opus")
                    .await
                    .map_err(ApplicationError::Storage)?;
                meditations
                    .set_storage_key(meditation.id(), &new_key)
                    .await
                    .map_err(ApplicationError::Repository)?;
            }
        }
    }
    store
        .delete_folder(&program.folder)
        .await
        .map_err(ApplicationError::Storage)?;
    Ok(())
}
