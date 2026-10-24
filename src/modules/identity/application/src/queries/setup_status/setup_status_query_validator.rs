use super::setup_status_query::SetupStatusQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct SetupStatusQueryValidator;

impl SetupStatusQueryValidator {
    pub fn validate(&self, _query: &SetupStatusQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "setup_status_query_validator.tests.rs"]
mod tests;
