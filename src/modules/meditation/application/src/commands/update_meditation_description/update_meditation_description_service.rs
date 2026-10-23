use std::sync::Arc;

use super::update_meditation_description::UpdateMeditationDescription;
use super::update_meditation_description_handler::UpdateMeditationDescriptionHandler;
use crate::ports::MeditationRepository;
use crate::services::validate_description;
use crate::ApplicationError;
use meditation_domain::Meditation;

pub struct UpdateMeditationDescriptionService {
    meditations: Arc<dyn MeditationRepository>,
}

impl UpdateMeditationDescriptionService {
    pub fn new(meditations: Arc<dyn MeditationRepository>) -> Self {
        Self { meditations }
    }
}

#[async_trait::async_trait]
impl UpdateMeditationDescriptionHandler for UpdateMeditationDescriptionService {
    async fn handle(
        &self,
        command: UpdateMeditationDescription,
    ) -> Result<Meditation, ApplicationError> {
        // Replaces the description; blank or `None` clears it.
        let UpdateMeditationDescription {
            meditation_id,
            description,
        } = command;
        let description = validate_description(description.as_deref())?;
        let meditation = self
            .meditations
            .find_by_id(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?
            .with_description(description.as_deref())
            .map_err(|_| ApplicationError::InvalidDescription)?;
        let updated = self
            .meditations
            .set_description(meditation_id, description.as_deref())
            .await
            .map_err(ApplicationError::Repository)?;
        if updated {
            Ok(meditation)
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "update_meditation_description_service.tests.rs"]
mod tests;
