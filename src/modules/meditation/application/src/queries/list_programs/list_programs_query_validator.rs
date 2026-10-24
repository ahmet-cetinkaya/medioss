use super::list_programs_query::ListProgramsQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListProgramsQueryValidator;

impl ListProgramsQueryValidator {
    pub fn validate(&self, _query: &ListProgramsQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_programs_query_validator.tests.rs"]
mod tests;
