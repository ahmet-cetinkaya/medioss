use super::list_recent_programs_query::ListRecentProgramsQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListRecentProgramsQueryValidator;

impl ListRecentProgramsQueryValidator {
    pub fn validate(&self, _query: &ListRecentProgramsQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_recent_programs_query_validator.tests.rs"]
mod tests;
