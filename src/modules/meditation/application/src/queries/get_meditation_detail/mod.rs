mod meditation_detail;

use crate::ports::{MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;

pub use self::meditation_detail::MeditationDetail;

/// Loads one meditation together with its home program (spec FR-008): the
/// Player borrows the program's cover and links to it by title, so the
/// detail read needs the program row, not just its id. `program` is `None`
/// when the meditation belongs to no program — or when its home program row
/// is gone, which degrades to a program-less detail instead of an error.
pub async fn get_meditation_detail<R, M, P>(
    repository: &R,
    memberships: &M,
    programs: &P,
    meditation_id: i64,
) -> Result<MeditationDetail, ApplicationError>
where
    R: MeditationRepository + ?Sized,
    M: MembershipRepository + ?Sized,
    P: ProgramRepository + ?Sized,
{
    let meditation = crate::queries::get_meditation(repository, meditation_id).await?;
    let program = match memberships
        .first_program_for_meditation(meditation.id())
        .await
        .map_err(ApplicationError::Repository)?
    {
        Some(program_id) => programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?,
        None => None,
    };
    Ok(MeditationDetail {
        meditation,
        program,
    })
}
