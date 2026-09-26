//! Removes a meditation from a program.
use crate::ports::{MembershipRepository, ProgramRepository};
use crate::ApplicationError;

/// Removes a membership; unknown pairs succeed idempotently (spec FR-027).
pub async fn remove_member<P, M>(
    programs: &P,
    memberships: &M,
    program_id: i64,
    meditation_id: i64,
) -> Result<(), ApplicationError>
where
    P: ProgramRepository + ?Sized,
    M: MembershipRepository + ?Sized,
{
    programs
        .find_by_id(program_id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?;
    memberships
        .remove(program_id, meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    Ok(())
}
