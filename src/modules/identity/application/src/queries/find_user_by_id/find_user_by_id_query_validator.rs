use super::find_user_by_id_query::FindUserByIdQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct FindUserByIdQueryValidator;

impl FindUserByIdQueryValidator {
    pub fn validate(&self, _query: &FindUserByIdQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "find_user_by_id_query_validator.tests.rs"]
mod tests;
