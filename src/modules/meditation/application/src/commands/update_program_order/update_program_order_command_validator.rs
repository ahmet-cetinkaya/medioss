use super::update_program_order_command::UpdateProgramOrderCommand;
use crate::ApplicationError;

/// The requested order can only be judged against the program's current members, so the handler owns that check.
#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateProgramOrderCommandValidator;

impl UpdateProgramOrderCommandValidator {
    pub fn validate(&self, _command: &UpdateProgramOrderCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "update_program_order_command_validator.tests.rs"]
mod tests;
