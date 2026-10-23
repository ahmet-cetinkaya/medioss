use std::sync::Arc;

use super::continue_program::ContinueProgram;
use super::get_continue_program::GetContinueProgram;
use super::get_continue_program_handler::GetContinueProgramHandler;
use crate::ports::{MembershipRepository, PlaybackStateRepository, ProgramRepository};
use crate::ApplicationError;

pub struct GetContinueProgramService {
    playback: Arc<dyn PlaybackStateRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
}

impl GetContinueProgramService {
    pub fn new(
        playback: Arc<dyn PlaybackStateRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            playback,
            memberships,
            programs,
        }
    }
}

#[async_trait::async_trait]
impl GetContinueProgramHandler for GetContinueProgramService {
    async fn handle(
        &self,
        query: GetContinueProgram,
    ) -> Result<Option<ContinueProgram>, ApplicationError> {
        let user_id = query.user_id;
        // Walk touches most-recent-first: a deleted meditation (no membership,
        // no program) must not permanently hide "continue" behind a live one
        // touched earlier.
        let touched = self
            .playback
            .list_touched(user_id)
            .await
            .map_err(ApplicationError::Repository)?;
        for (meditation_id, _touched_at) in touched {
            let Some(program_id) = self
                .memberships
                .first_program_for_meditation(meditation_id)
                .await
                .map_err(ApplicationError::Repository)?
            else {
                continue;
            };
            let Some(program) = self
                .programs
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
}

#[cfg(test)]
#[path = "get_continue_program_service.tests.rs"]
mod tests;
