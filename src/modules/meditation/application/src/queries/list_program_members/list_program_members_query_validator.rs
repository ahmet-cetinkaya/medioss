use super::list_program_members_query::ListProgramMembersQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListProgramMembersQueryValidator;

impl ListProgramMembersQueryValidator {
    pub fn validate(&self, _query: &ListProgramMembersQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_program_members_query_validator.tests.rs"]
mod tests;
