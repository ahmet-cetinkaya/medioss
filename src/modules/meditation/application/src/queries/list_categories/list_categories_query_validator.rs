use super::list_categories_query::ListCategoriesQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListCategoriesQueryValidator;

impl ListCategoriesQueryValidator {
    pub fn validate(&self, _query: &ListCategoriesQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_categories_query_validator.tests.rs"]
mod tests;
