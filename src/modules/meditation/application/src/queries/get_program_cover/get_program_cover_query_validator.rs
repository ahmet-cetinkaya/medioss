use super::get_program_cover_query::GetProgramCoverQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct GetProgramCoverQueryValidator;

impl GetProgramCoverQueryValidator {
    pub fn validate(&self, _query: &GetProgramCoverQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "get_program_cover_query_validator.tests.rs"]
mod tests;
