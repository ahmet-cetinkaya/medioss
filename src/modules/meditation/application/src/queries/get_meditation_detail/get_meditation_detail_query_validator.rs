use super::get_meditation_detail_query::GetMeditationDetailQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetMeditationDetailQueryValidator;

impl GetMeditationDetailQueryValidator {
    pub fn validate(&self, _query: &GetMeditationDetailQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "get_meditation_detail_query_validator.tests.rs"]
mod tests;
