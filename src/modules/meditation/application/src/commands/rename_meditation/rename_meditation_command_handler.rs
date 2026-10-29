use std::sync::Arc;

use super::rename_meditation_command::RenameMeditationCommand;
use super::rename_meditation_command_validator::RenameMeditationCommandValidator;
use crate::ports::{CommandHandler, MeditationRepository};
use crate::ApplicationError;
use meditation_domain::Meditation;

pub struct RenameMeditationCommandHandler {
    meditations: Arc<dyn MeditationRepository>,
    validator: RenameMeditationCommandValidator,
}

impl RenameMeditationCommandHandler {
    pub fn new(meditations: Arc<dyn MeditationRepository>) -> Self {
        Self {
            meditations,
            validator: RenameMeditationCommandValidator,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<RenameMeditationCommand> for RenameMeditationCommandHandler {
    type Output = Meditation;

    async fn handle(
        &self,
        command: RenameMeditationCommand,
    ) -> Result<Meditation, ApplicationError> {
        self.validator.validate(&command)?;
        let name = self.validator.normalized_name(&command);
        let meditation_id = command.meditation_id;
        let renamed = self
            .meditations
            .find_by_id(meditation_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?
            .with_name(name)
            .map_err(|_| ApplicationError::InvalidInput("name must be 1-200 characters"))?;
        let updated = self
            .meditations
            .set_name(meditation_id, name)
            .await
            .map_err(ApplicationError::Repository)?;
        if updated {
            Ok(renamed)
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}

#[cfg(test)]
#[path = "rename_meditation_command_handler.tests.rs"]
mod tests;
