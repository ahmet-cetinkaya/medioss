use std::sync::Arc;

use super::favorite_meditation_command::FavoriteMeditationCommand;
use super::favorite_meditation_command_validator::FavoriteMeditationCommandValidator;
use crate::ports::CommandHandler;
use crate::ports::{Clock, FavoriteRepository, MeditationRepository};
use crate::services::ensure_meditation_exists;
use crate::ApplicationError;

/// Idempotent: favoriting an already-favorite meditation succeeds unchanged.
pub struct FavoriteMeditationCommandHandler {
    validator: FavoriteMeditationCommandValidator,
    meditations: Arc<dyn MeditationRepository>,
    favorites: Arc<dyn FavoriteRepository>,
    clock: Arc<dyn Clock>,
}

impl FavoriteMeditationCommandHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        favorites: Arc<dyn FavoriteRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            validator: FavoriteMeditationCommandValidator,
            meditations,
            favorites,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<FavoriteMeditationCommand> for FavoriteMeditationCommandHandler {
    type Output = ();

    async fn handle(&self, command: FavoriteMeditationCommand) -> Result<(), ApplicationError> {
        self.validator.validate(&command)?;
        ensure_meditation_exists(self.meditations.as_ref(), command.meditation_id).await?;
        let now = self.clock.now().await;
        self.favorites
            .add(command.user_id, command.meditation_id, now)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "favorite_meditation_command_handler.tests.rs"]
mod tests;
