use super::record_position_command::RecordPositionCommand;
use super::record_position_command::MAX_POSITION_MS;
use crate::ApplicationError;

/// A reported position must lie within `0..=MAX_POSITION_MS`.
#[derive(Debug, Default, Clone, Copy)]
pub struct RecordPositionCommandValidator;

impl RecordPositionCommandValidator {
    pub fn validate(&self, command: &RecordPositionCommand) -> Result<(), ApplicationError> {
        if !(0..=MAX_POSITION_MS).contains(&command.position_ms) {
            return Err(ApplicationError::InvalidInput(
                "position must be between 0 and 30 days in milliseconds",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "record_position_command_validator.tests.rs"]
mod tests;
