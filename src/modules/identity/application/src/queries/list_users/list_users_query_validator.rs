use super::list_users_query::ListUsersQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListUsersQueryValidator;

impl ListUsersQueryValidator {
    pub fn validate(&self, _query: &ListUsersQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_users_query_validator.tests.rs"]
mod tests;
