use crate::ports::{MembershipRepository, PlaybackStateRepository, Program, ProgramRepository};
use crate::ApplicationError;
use kernel::UserId;

pub struct ContinueProgram {
    pub program: Program,
    pub meditation_id: i64,
}

pub async fn get_continue_program<K, M, P>(
    playback: &K,
    memberships: &M,
    programs: &P,
    user_id: UserId,
) -> Result<Option<ContinueProgram>, ApplicationError>
where
    K: PlaybackStateRepository + ?Sized,
    M: MembershipRepository + ?Sized,
    P: ProgramRepository + ?Sized,
{
    let Some(meditation_id) = playback
        .most_recent_meditation_id(user_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Ok(None);
    };
    let Some(program_id) = memberships
        .first_program_for_meditation(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Ok(None);
    };
    let Some(program) = programs
        .find_by_id(program_id)
        .await
        .map_err(ApplicationError::Repository)?
    else {
        return Ok(None);
    };
    Ok(Some(ContinueProgram {
        program,
        meditation_id,
    }))
}
