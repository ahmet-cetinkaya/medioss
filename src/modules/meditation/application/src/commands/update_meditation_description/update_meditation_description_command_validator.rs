use super::update_meditation_description_command::UpdateMeditationDescriptionCommand;
use crate::services::validate_description;
use crate::ApplicationError;

/// A description is trimmed (blank clears it) and capped at the domain length.
#[derive(Debug, Default, Clone, Copy)]
pub struct UpdateMeditationDescriptionCommandValidator;

impl UpdateMeditationDescriptionCommandValidator {
    pub fn validate(
        &self,
        command: &UpdateMeditationDescriptionCommand,
    ) -> Result<(), ApplicationError> {
        self.normalized_description(command).map(|_| ())
    }

    pub fn normalized_description(
        &self,
        command: &UpdateMeditationDescriptionCommand,
    ) -> Result<Option<String>, ApplicationError> {
        validate_description(command.description.as_deref())
    }
}

#[cfg(test)]
#[path = "update_meditation_description_command_validator.tests.rs"]
mod tests;
