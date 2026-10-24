use super::update_mix_command::UpdateMixCommand;
use crate::services::validate_mix_items;
use crate::ApplicationError;
use soundscape_domain::valid_name;

#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateMixCommandValidator;

impl UpdateMixCommandValidator {
    pub fn validate(&self, command: &UpdateMixCommand) -> Result<(), ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("mix name"))?;
        validate_mix_items(&command.items)
    }

    pub fn normalized_name(&self, command: &UpdateMixCommand) -> String {
        command.name.trim().into()
    }
}

#[cfg(test)]
#[path = "update_mix_command_validator.tests.rs"]
mod tests;
