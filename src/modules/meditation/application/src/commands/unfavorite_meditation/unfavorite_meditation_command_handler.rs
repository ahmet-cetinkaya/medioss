use std::sync::Arc;

use super::unfavorite_meditation_command::UnfavoriteMeditationCommand;
use super::unfavorite_meditation_command_validator::UnfavoriteMeditationCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{FavoriteRepository, MeditationRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

/// Idempotent: unfavoriting a meditation that is not a favorite succeeds, but
/// an unknown meditation is still `NotFound`.
pub struct UnfavoriteMeditationCommandHandler {
    validator: UnfavoriteMeditationCommandValidator,
    meditations: Arc<dyn MeditationRepository>,
    favorites: Arc<dyn FavoriteRepository>,
}

impl UnfavoriteMeditationCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        favorites: Arc<dyn FavoriteRepository>,
    ) -> Self {
        Self {
            validator: UnfavoriteMeditationCommandValidator,
            meditations,
            favorites,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<UnfavoriteMeditationCommand> for UnfavoriteMeditationCommandHandler {
    type Output = ();

    async fn handle(&self, command: UnfavoriteMeditationCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        ensure_meditation_exists(self.meditations.as_ref(), command.meditation_id).await?;
        self.favorites
            .remove(command.user_id, command.meditation_id)
            .await
            .map(|_was_favorite| ())
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "unfavorite_meditation_command_handler.tests.rs"]
mod tests;
