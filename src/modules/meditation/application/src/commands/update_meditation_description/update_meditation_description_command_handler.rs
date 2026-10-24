use std::sync::Arc;

use super::update_meditation_description_command::UpdateMeditationDescriptionCommand;
use super::update_meditation_description_command_validator::UpdateMeditationDescriptionCommandValidator;
use crate::ports::{CommandHandler, MeditationRepository};
use crate::ApplicationError;
use meditation_domain::Meditation;

pub struct UpdateMeditationDescriptionCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    validator: UpdateMeditationDescriptionCommandValidator,
}

impl UpdateMeditationDescriptionCommandHandler {
    pub fn new(meditations: Arc<dyn MeditationRepository>) -> Self {
        Self {
            meditations,
            validator: UpdateMeditationDescriptionCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UpdateMeditationDescriptionCommand>
    for UpdateMeditationDescriptionCommandHandler
{
    type Output = Meditation;

    async fn handle(
        &self,
        command: UpdateMeditationDescriptionCommand,
    ) -> Result<Meditation, ApplicationError> {
        self.validator.validate(&command)?;

        // Replaces the description; blank or `None` clears it.
        let description = self.validator.normalized_description(&command)?;
        let meditation_id = command.meditation_id;
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
#[path = "update_meditation_description_command_handler.tests.rs"]
mod tests;
