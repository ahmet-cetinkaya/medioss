use super::get_continue_program_query::GetContinueProgramQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetContinueProgramQueryValidator;

impl GetContinueProgramQueryValidator {
    pub fn validate(&self, _query: &GetContinueProgramQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "get_continue_program_query_validator.tests.rs"]
mod tests;
