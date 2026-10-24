use super::record_completion_command::RecordCompletionCommand;
use crate::ApplicationError;

/// A session cannot finish before it started.
#[derive(Debug, Default, Clone, Copy)]
pub struct RecordCompletionCommandValidator;

impl RecordCompletionCommandValidator {
    pub fn validate(&self, command: &RecordCompletionCommand) -> Result<(), ApplicationError> {
        if command.finished_at < command.started_at {
            return Err(ApplicationError::InvalidInput(
                "finished_at must not precede started_at",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "record_completion_command_validator.tests.rs"]
mod tests;
