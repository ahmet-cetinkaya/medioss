use std::sync::Arc;

use super::list_recent_programs::ListRecentPrograms;
use super::list_recent_programs_handler::ListRecentProgramsHandler;
use super::recent_program::RecentProgram;
use crate::ports::{MembershipRepository, PlaybackStateRepository, ProgramRepository};
use crate::ApplicationError;

pub struct ListRecentProgramsService {
    playback: Arc<dyn PlaybackStateRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
}

impl ListRecentProgramsService {
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
impl ListRecentProgramsHandler for ListRecentProgramsService {
    async fn handle(
        &self,
        query: ListRecentPrograms,
    ) -> Result<Vec<RecentProgram>, ApplicationError> {
        let ListRecentPrograms { user_id, limit } = query;
        let touched = self
            .playback
            .list_touched(user_id)
            .await
            .map_err(ApplicationError::Repository)?;

        let mut seen_programs = std::collections::HashSet::new();
        let mut recent = Vec::new();
        for (meditation_id, last_touched_at) in touched {
            let Some(program_id) = self
                .memberships
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
            if let Some(program) = self
                .programs
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
}

#[cfg(test)]
#[path = "list_recent_programs_service.tests.rs"]
mod tests;
