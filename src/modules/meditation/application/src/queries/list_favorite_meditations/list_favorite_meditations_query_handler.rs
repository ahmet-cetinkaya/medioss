use std::sync::Arc;

use meditation_domain::Meditation;

use super::list_favorite_meditations_query::ListFavoriteMeditationsQuery;
use super::list_favorite_meditations_query_validator::ListFavoriteMeditationsQueryValidator;
use crate::ports::QueryHandler;
use crate::ports::{FavoriteRepository, MeditationRepository};
use crate::ApplicationError;

/// The caller's favorite meditations, most recently favorited first.
pub struct ListFavoriteMeditationsQueryHandler {
    validator: ListFavoriteMeditationsQueryValidator,
    meditations: Arc<dyn MeditationRepository>,
    favorites: Arc<dyn FavoriteRepository>,
}

impl ListFavoriteMeditationsQueryHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        favorites: Arc<dyn FavoriteRepository>,
    ) -> Self {
        Self {
            validator: ListFavoriteMeditationsQueryValidator,
            meditations,
            favorites,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListFavoriteMeditationsQuery> for ListFavoriteMeditationsQueryHandler {
    type Output = Vec<Meditation>;

    async fn handle(
        &self,
        query: ListFavoriteMeditationsQuery,
    ) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        let ordered_ids = self
            .favorites
            .list_for_user(query.user_id)
            .await
            .map_err(ApplicationError::Repository)?;
        let mut library = self
            .meditations
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        // Favorite rows are not cascaded away on every path, so a favorite
        // whose meditation is gone is skipped rather than failing the list.
        let mut listed = Vec::with_capacity(ordered_ids.len());
        for id in ordered_ids {
            if let Some(position) = library.iter().position(|meditation| meditation.id() == id) {
                listed.push(library.swap_remove(position));
            }
        }
        Ok(listed)
    }
}

#[cfg(test)]
#[path = "list_favorite_meditations_query_handler.tests.rs"]
mod tests;
