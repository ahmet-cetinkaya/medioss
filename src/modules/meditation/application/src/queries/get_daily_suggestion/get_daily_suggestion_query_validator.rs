use super::get_daily_suggestion_query::GetDailySuggestionQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetDailySuggestionQueryValidator;

impl GetDailySuggestionQueryValidator {
    pub fn validate(&self, _query: &GetDailySuggestionQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "get_daily_suggestion_query_validator.tests.rs"]
mod tests;
