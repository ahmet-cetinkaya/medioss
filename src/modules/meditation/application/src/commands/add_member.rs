//! Adds a meditation to a program.
use crate::commands::ensure_meditation_exists;
use crate::ports::{Clock, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

/// Adds a meditation to a program; both must exist, repeats are idempotent
/// (spec FR-027).
pub async fn add_member<R, P, M, K>(
    meditations: &R,
    programs: &P,
    memberships: &M,
    clock: &K,
    program_id: i64,
    meditation_id: i64,
) -> Result<(), ApplicationError>
where
    R: MeditationRepository,
    P: ProgramRepository,
    M: MembershipRepository,
    K: Clock,
{
    ensure_meditation_exists(meditations, meditation_id).await?;
    programs
        .find_by_id(program_id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?;
    memberships
        .add(program_id, meditation_id, clock.now().await)
        .await;
    Ok(())
}
