use super::remove_member_command::RemoveMemberCommand;
use crate::ApplicationError;

/// The command carries only identifiers; existence is checked against the
/// repositories in the handler, so there is no input rule to enforce here.
#[derive(Debug, Default, Clone, Copy)]
pub struct RemoveMemberCommandValidator;

impl RemoveMemberCommandValidator {
    pub fn validate(&self, _command: &RemoveMemberCommand) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "remove_member_command_validator.tests.rs"]
mod tests;
