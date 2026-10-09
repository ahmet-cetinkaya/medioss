//! Reorders a program's meditations.
use crate::ports::{MembershipRepository, ProgramRepository};
use crate::ApplicationError;

/// Rewrites the program's order. `meditation_ids` must be exactly the
/// program's current meditations, each once; anything else is rejected.
pub async fn update_program_order<P, M>(
    programs: &P,
    memberships: &M,
    program_id: i64,
    meditation_ids: &[i64],
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
    let mut current: Vec<i64> = memberships
        .list_by_program(program_id)
        .await
        .map_err(ApplicationError::Repository)?
        .into_iter()
        .map(|member| member.meditation_id)
        .collect();
    let mut requested = meditation_ids.to_vec();
    current.sort_unstable();
    requested.sort_unstable();
    if current != requested {
        return Err(ApplicationError::InvalidProgramOrder);
    }
    memberships
        .set_order(program_id, meditation_ids)
        .await
        .map_err(ApplicationError::Repository)
}
