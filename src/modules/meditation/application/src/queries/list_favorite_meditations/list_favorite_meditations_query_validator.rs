use super::list_favorite_meditations_query::ListFavoriteMeditationsQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListFavoriteMeditationsQueryValidator;

impl ListFavoriteMeditationsQueryValidator {
    pub fn validate(&self, _query: &ListFavoriteMeditationsQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_favorite_meditations_query_validator.tests.rs"]
mod tests;
