use super::unfavorite_meditation_command::UnfavoriteMeditationCommand;
use crate::ApplicationError;

/// The command carries no input rule beyond the meditation lookup the handler
/// performs; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct UnfavoriteMeditationCommandValidator;

impl UnfavoriteMeditationCommandValidator {
    pub fn validate(&self, _command: &UnfavoriteMeditationCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "unfavorite_meditation_command_validator.tests.rs"]
mod tests;
