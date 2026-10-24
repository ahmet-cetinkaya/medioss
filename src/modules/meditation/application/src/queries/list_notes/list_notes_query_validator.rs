use super::list_notes_query::ListNotesQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListNotesQueryValidator;

impl ListNotesQueryValidator {
    pub fn validate(&self, _query: &ListNotesQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_notes_query_validator.tests.rs"]
mod tests;
