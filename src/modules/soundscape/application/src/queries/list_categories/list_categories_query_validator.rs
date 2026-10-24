use super::list_categories_query::ListCategoriesQuery;
use crate::ApplicationError;

/// These queries carry no input to check, so every query is accepted.
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
