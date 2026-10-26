use super::favorite_meditation_command::FavoriteMeditationCommand;
use crate::ApplicationError;

/// The command carries no input rule beyond the meditation lookup the handler
/// performs; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct FavoriteMeditationCommandValidator;

impl FavoriteMeditationCommandValidator {
    pub fn validate(&self, _command: &FavoriteMeditationCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "favorite_meditation_command_validator.tests.rs"]
mod tests;
