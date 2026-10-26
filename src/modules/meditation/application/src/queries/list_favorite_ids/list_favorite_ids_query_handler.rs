use std::collections::HashSet;
use std::sync::Arc;

use super::list_favorite_ids_query::ListFavoriteIdsQuery;
use super::list_favorite_ids_query_validator::ListFavoriteIdsQueryValidator;
use crate::ports::FavoriteRepository;
use crate::ports::QueryHandler;
use crate::ApplicationError;

/// The caller's favorite identifiers as a set: one read decorates a whole list.
pub struct ListFavoriteIdsQueryHandler {
    validator: ListFavoriteIdsQueryValidator,
    favorites: Arc<dyn FavoriteRepository>,
}

impl ListFavoriteIdsQueryHandler {
    pub fn new(favorites: Arc<dyn FavoriteRepository>) -> Self {
        Self {
            validator: ListFavoriteIdsQueryValidator,
            favorites,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListFavoriteIdsQuery> for ListFavoriteIdsQueryHandler {
    type Output = HashSet<i64>;

    async fn handle(&self, query: ListFavoriteIdsQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        self.favorites
            .ids_for_user(query.user_id)
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_favorite_ids_query_handler.tests.rs"]
mod tests;
