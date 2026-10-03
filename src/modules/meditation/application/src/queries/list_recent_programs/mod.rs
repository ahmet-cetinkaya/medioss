mod recent_program;

use crate::ports::{MembershipRepository, PlaybackStateRepository, ProgramRepository};
use crate::ApplicationError;

use kernel::UserId;

pub use self::recent_program::RecentProgram;

pub async fn list_recent_programs<K, M, P>(
    playback: &K,
    memberships: &M,
    programs: &P,
    user_id: UserId,
    limit: Option<usize>,
) -> Result<Vec<RecentProgram>, ApplicationError>
where
    K: PlaybackStateRepository + ?Sized,
    M: MembershipRepository + ?Sized,
    P: ProgramRepository + ?Sized,
{
    let touched = playback
        .list_touched(user_id)
        .await
        .map_err(ApplicationError::Repository)?;

    let mut seen_programs = std::collections::HashSet::new();
    let mut recent = Vec::new();
    for (meditation_id, last_touched_at) in touched {
        let Some(program_id) = memberships
            .first_program_for_meditation(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
        else {
            continue;
        };
        // A program can hold several touched meditations: only its first
        // (most recent) touch should surface it in the list.
        if !seen_programs.insert(program_id) {
            continue;
        }
        if let Some(program) = programs
            .find_by_id(program_id)
            .await
            .map_err(ApplicationError::Repository)?
        {
            recent.push(RecentProgram {
                program,
                last_touched_at,
            });
        }
        if limit.is_some_and(|limit| recent.len() >= limit) {
            break;
        }
    }
    Ok(recent)
}
