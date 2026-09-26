//! Deletes a program: memberships and cover bytes go, meditations stay.
use crate::ports::{BlobStore, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

/// Deletes a program (spec FR-028). Note: memberships are removed through
/// the program's membership sweep, mirroring the repository contract.
pub async fn delete_program<P, M, S>(
    programs: &P,
    memberships: &M,
    store: &S,
    program_id: i64,
) -> Result<(), ApplicationError>
where
    P: ProgramRepository,
    M: MembershipRepository,
    S: BlobStore,
{
    let Some(program) = programs
        .delete(program_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Err(ApplicationError::NotFound);
    };
    // The fake and SeaORM repositories expose remove_for_meditation; a
    // program-scoped sweep is expressed as listing and removing each pair
    // so the port stays minimal.
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
    if let Some(key) = program.cover_storage_key {
        store
            .delete(&key)
            .await
            .map_err(ApplicationError::Storage)?;
    }
    Ok(())
}
