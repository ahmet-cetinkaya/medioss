use super::list_favorite_ids_query::ListFavoriteIdsQuery;
use crate::ApplicationError;

/// The query carries no input to check; the validator keeps every use case the same shape.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListFavoriteIdsQueryValidator;

impl ListFavoriteIdsQueryValidator {
    pub fn validate(&self, _query: &ListFavoriteIdsQuery) -> Result<(), ApplicationError> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "list_favorite_ids_query_validator.tests.rs"]
mod tests;
