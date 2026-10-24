use std::sync::Arc;

use super::get_meditation::GetMeditation;
use super::get_meditation_handler::GetMeditationHandler;
use crate::ports::MeditationRepository;
use crate::services::load_meditation;
use crate::ApplicationError;
use meditation_domain::Meditation;

pub struct GetMeditationService {
    meditations: Arc<dyn MeditationRepository>,
}

impl GetMeditationService {
    pub fn new(meditations: Arc<dyn MeditationRepository>) -> Self {
        Self { meditations }
    }
}

#[async_trait::async_trait]
impl GetMeditationHandler for GetMeditationService {
    async fn handle(&self, query: GetMeditation) -> Result<Meditation, ApplicationError> {
        // Loads one meditation (spec FR-008).
        load_meditation(self.meditations.as_ref(), query.meditation_id).await
    }
}

#[cfg(test)]
#[path = "get_meditation_service.tests.rs"]
mod tests;
