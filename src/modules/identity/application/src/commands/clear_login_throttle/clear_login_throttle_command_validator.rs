use super::clear_login_throttle_command::ClearLoginThrottleCommand;
use crate::models::normalize_login_identifier;
use crate::ApplicationError;

/// Input rules of [`ClearLoginThrottleCommand`]. Its fields are already valid by
/// construction (domain value types), so no rule rejects a command yet; it owns the identifier's canonical form.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClearLoginThrottleCommandValidator;

impl ClearLoginThrottleCommandValidator {
    pub fn validate(&self, _command: &ClearLoginThrottleCommand) -> Result<(), ApplicationError> {
        Ok(())
    }

    /// The identifier throttle rows are keyed on.
    pub fn normalized_identifier(&self, command: &ClearLoginThrottleCommand) -> String {
        normalize_login_identifier(&command.identifier)
    }
}

#[cfg(test)]
#[path = "clear_login_throttle_command_validator.tests.rs"]
mod tests;
