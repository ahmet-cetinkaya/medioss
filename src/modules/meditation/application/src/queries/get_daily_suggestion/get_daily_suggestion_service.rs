use std::sync::Arc;

use super::daily_suggestion::DailySuggestion;
use super::get_daily_suggestion::GetDailySuggestion;
use super::get_daily_suggestion_handler::GetDailySuggestionHandler;
use crate::ports::{Clock, MeditationRepository, MembershipRepository, ProgramRepository};
use crate::ApplicationError;
use chrono::Datelike;

pub struct GetDailySuggestionService {
    meditations: Arc<dyn MeditationRepository>,
    clock: Arc<dyn Clock>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
}

impl GetDailySuggestionService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        clock: Arc<dyn Clock>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            meditations,
            clock,
            memberships,
            programs,
        }
    }
}

#[async_trait::async_trait]
impl GetDailySuggestionHandler for GetDailySuggestionService {
    async fn handle(
        &self,
        _query: GetDailySuggestion,
    ) -> Result<Option<DailySuggestion>, ApplicationError> {
        let mut library = self
            .meditations
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        if library.is_empty() {
            return Ok(None);
        }
        // Stable order regardless of the repository's own ordering, so the
        // day's pick does not shift if newer rows are inserted later today.
        library.sort_by_key(|meditation| meditation.id());
        let day = self.clock.now().await.num_days_from_ce();
        let index = (day.rem_euclid(library.len() as i32)) as usize;
        let Some(meditation) = library.into_iter().nth(index) else {
            return Ok(None);
        };
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
        Ok(Some(DailySuggestion {
            meditation,
            program,
        }))
    }
}

#[cfg(test)]
#[path = "get_daily_suggestion_service.tests.rs"]
mod tests;
