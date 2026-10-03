mod continue_program;

use crate::ports::{MembershipRepository, PlaybackStateRepository, ProgramRepository};
use crate::ApplicationError;
use kernel::UserId;

pub use self::continue_program::ContinueProgram;

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
    // Walk touches most-recent-first: a deleted meditation (no membership,
    // no program) must not permanently hide "continue" behind a live one
    // touched earlier.
    let touched = playback
        .list_touched(user_id)
        .await
        .map_err(ApplicationError::Repository)?;
    for (meditation_id, _touched_at) in touched {
        let Some(program_id) = memberships
            .first_program_for_meditation(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
        else {
            continue;
        };
        let Some(program) = programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
        else {
            continue;
        };
        return Ok(Some(ContinueProgram {
            program,
            meditation_id,
        }));
    }
    Ok(None)
}
