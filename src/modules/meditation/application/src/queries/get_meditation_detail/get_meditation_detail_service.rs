use std::sync::Arc;

use super::get_meditation_detail::GetMeditationDetail;
use super::get_meditation_detail_handler::GetMeditationDetailHandler;
use super::meditation_detail::MeditationDetail;
use crate::ports::{MeditationRepository, MembershipRepository, ProgramRepository};
use crate::services::load_meditation;
use crate::ApplicationError;

pub struct GetMeditationDetailService {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
}

impl GetMeditationDetailService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            meditations,
            memberships,
            programs,
        }
    }
}

#[async_trait::async_trait]
impl GetMeditationDetailHandler for GetMeditationDetailService {
    async fn handle(
        &self,
        query: GetMeditationDetail,
    ) -> Result<MeditationDetail, ApplicationError> {
        // Loads one meditation together with its home program (spec FR-008):
        // the Player borrows the program's cover and links to it by title, so
        // the detail read needs the program row, not just its id. `program` is
        // `None` when the meditation belongs to no program — or when its home
        // program row is gone, which degrades to a program-less detail instead
        // of an error.
        let meditation = load_meditation(self.meditations.as_ref(), query.meditation_id).await?;
        let program = match self
            .memberships
            .first_program_for_meditation(meditation.id())
            .await
            .map_err(ApplicationError::Repository)?
        {
            Some(program_id) => self
                .programs
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
}

#[cfg(test)]
#[path = "get_meditation_detail_service.tests.rs"]
mod tests;
