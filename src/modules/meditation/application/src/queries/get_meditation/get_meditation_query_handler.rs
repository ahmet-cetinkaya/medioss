use std::sync::Arc;

use super::get_meditation_query::GetMeditationQuery;
use super::get_meditation_query_validator::GetMeditationQueryValidator;
use crate::ports::MeditationRepository;
use crate::ports::QueryHandler;
use crate::services::load_meditation;
use crate::ApplicationError;
use meditation_domain::Meditation;

pub struct GetMeditationQueryHandler {
    meditations: Arc<dyn MeditationRepository>,
    validator: GetMeditationQueryValidator,
}

impl GetMeditationQueryHandler {
    pub fn new(meditations: Arc<dyn MeditationRepository>) -> Self {
        Self {
            meditations,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<GetMeditationQuery> for GetMeditationQueryHandler {
    type Output = Meditation;

    async fn handle(&self, query: GetMeditationQuery) -> Result<Meditation, ApplicationError> {
        self.validator.validate(&query)?;
        // Loads one meditation (spec FR-008).
        load_meditation(self.meditations.as_ref(), query.meditation_id).await
    }
}

#[cfg(test)]
#[path = "get_meditation_query_handler.tests.rs"]
mod tests;
