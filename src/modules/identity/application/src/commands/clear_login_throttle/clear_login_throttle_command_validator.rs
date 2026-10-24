use super::clear_login_throttle_command::ClearLoginThrottleCommand;
use crate::ApplicationError;

/// Input rules of [`ClearLoginThrottleCommand`]. Its fields are already valid by
/// construction (domain value types), so no rule rejects a command yet.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClearLoginThrottleCommandValidator;

impl ClearLoginThrottleCommandValidator {
    pub fn validate(&self, _command: &ClearLoginThrottleCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "clear_login_throttle_command_validator.tests.rs"]
mod tests;
