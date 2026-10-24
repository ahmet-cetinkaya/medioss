use super::add_member_command::AddMemberCommand;
use crate::ApplicationError;

/// The command carries only identifiers; existence is checked against the
/// repositories in the handler, so there is no input rule to enforce here.
#[derive(Debug, Default, Clone, Copy)]
pub struct AddMemberCommandValidator;

impl AddMemberCommandValidator {
    pub fn validate(&self, _command: &AddMemberCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "add_member_command_validator.tests.rs"]
mod tests;
