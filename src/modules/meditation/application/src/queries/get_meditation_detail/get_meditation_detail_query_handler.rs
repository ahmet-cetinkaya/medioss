use std::sync::Arc;

use super::get_meditation_detail_query::GetMeditationDetailQuery;
use super::get_meditation_detail_query_validator::GetMeditationDetailQueryValidator;
use super::meditation_detail::MeditationDetail;
use crate::ports::{MeditationRepository, MembershipRepository, ProgramRepository, QueryHandler};
use crate::services::load_meditation;
use crate::ApplicationError;

pub struct GetMeditationDetailQueryHandler {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
    validator: GetMeditationDetailQueryValidator,
}

impl GetMeditationDetailQueryHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            meditations,
            memberships,
            programs,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<GetMeditationDetailQuery> for GetMeditationDetailQueryHandler {
    type Output = MeditationDetail;

    async fn handle(
        &self,
        query: GetMeditationDetailQuery,
    ) -> Result<MeditationDetail, ApplicationError> {
        self.validator.validate(&query)?;
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
#[path = "get_meditation_detail_query_handler.tests.rs"]
mod tests;
