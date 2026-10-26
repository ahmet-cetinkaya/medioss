use std::collections::HashMap;
use std::sync::Arc;

use super::favorite_entry::FavoriteEntry;
use super::favorite_program::FavoriteProgram;
use super::list_favorite_meditations_query::ListFavoriteMeditationsQuery;
use super::list_favorite_meditations_query_validator::ListFavoriteMeditationsQueryValidator;
use crate::ports::QueryHandler;
use crate::ports::{
    FavoriteRepository, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::ApplicationError;

/// The caller's favorite meditations, most recently favorited first, each
/// with the programs it belongs to.
pub struct ListFavoriteMeditationsQueryHandler {
    validator: ListFavoriteMeditationsQueryValidator,
    meditations: Arc<dyn MeditationRepository>,
    favorites: Arc<dyn FavoriteRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
}

impl ListFavoriteMeditationsQueryHandler {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        favorites: Arc<dyn FavoriteRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
    ) -> Self {
        Self {
            validator: ListFavoriteMeditationsQueryValidator,
            meditations,
            favorites,
            memberships,
            programs,
        }
    }

    /// Meditation id -> the programs it belongs to, earliest membership first.
    async fn programs_by_meditation(
        &self,
    ) -> Result<HashMap<i64, Vec<FavoriteProgram>>, ApplicationError> {
        let mut joined = Vec::new();
        for program in self
            .programs
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?
        {
            let members = self
                .memberships
                .list_by_program(program.id)
                .await
                .map_err(ApplicationError::Repository)?;
            for membership in members {
                let label = FavoriteProgram {
                    id: program.id,
                    title: program.title.clone(),
                };
                joined.push((membership.added_at, membership.meditation_id, label));
            }
        }
        joined.sort_by_key(|(added_at, _, label)| (*added_at, label.id));
        let mut grouped: HashMap<i64, Vec<FavoriteProgram>> = HashMap::new();
        for (_, meditation_id, label) in joined {
            grouped.entry(meditation_id).or_default().push(label);
        }
        Ok(grouped)
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListFavoriteMeditationsQuery> for ListFavoriteMeditationsQueryHandler {
    type Output = Vec<FavoriteEntry>;

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
        let mut programs = self.programs_by_meditation().await?;
        let mut listed = Vec::with_capacity(ordered_ids.len());
        for id in ordered_ids {
            if let Some(position) = library.iter().position(|meditation| meditation.id() == id) {
                listed.push(FavoriteEntry {
                    meditation: library.swap_remove(position),
                    programs: programs.remove(&id).unwrap_or_default(),
                });
            }
        }
        Ok(listed)
    }
}

#[cfg(test)]
#[path = "list_favorite_meditations_query_handler.tests.rs"]
mod tests;
