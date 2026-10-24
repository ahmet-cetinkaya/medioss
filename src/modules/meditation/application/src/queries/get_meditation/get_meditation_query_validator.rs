use super::get_meditation_query::GetMeditationQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMeditationQueryValidator;

impl GetMeditationQueryValidator {
    pub fn validate(&self, _query: &GetMeditationQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "get_meditation_query_validator.tests.rs"]
mod tests;
