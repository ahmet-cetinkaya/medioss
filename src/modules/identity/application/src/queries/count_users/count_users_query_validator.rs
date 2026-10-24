use super::count_users_query::CountUsersQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct CountUsersQueryValidator;

impl CountUsersQueryValidator {
    pub fn validate(&self, _query: &CountUsersQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "count_users_query_validator.tests.rs"]
mod tests;
