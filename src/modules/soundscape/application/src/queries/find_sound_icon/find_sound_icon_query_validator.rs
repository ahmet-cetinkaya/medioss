use super::find_sound_icon_query::FindSoundIconQuery;
use crate::ApplicationError;

/// These queries carry no input to check, so every query is accepted.
#[derive(Debug, Default, Clone, Copy)]
pub struct FindSoundIconQueryValidator;

impl FindSoundIconQueryValidator {
    pub fn validate(&self, _query: &FindSoundIconQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "find_sound_icon_query_validator.tests.rs"]
mod tests;
