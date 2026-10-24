use std::sync::Arc;

use super::list_library::ListLibrary;
use super::list_library_handler::{LibraryEntry, ListLibraryHandler};
use super::matches::matches;
use super::search_index::SearchIndex;
use crate::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::ApplicationError;

pub struct ListLibraryService {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
    categories: Arc<dyn CategoryRepository>,
}

impl ListLibraryService {
    pub fn new(
        meditations: Arc<dyn MeditationRepository>,
        memberships: Arc<dyn MembershipRepository>,
        programs: Arc<dyn ProgramRepository>,
        categories: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            meditations,
            memberships,
            programs,
            categories,
        }
    }
}

#[async_trait::async_trait]
impl ListLibraryHandler for ListLibraryService {
    async fn handle(&self, query: ListLibrary) -> Result<Vec<LibraryEntry>, ApplicationError> {
        let library = self
            .meditations
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        let query = query.query.as_deref();
        let is_blank = query.map(str::trim).is_none_or(str::is_empty);
        let library = if is_blank {
            library
        } else {
            let query = query.unwrap_or_default();
            let index = SearchIndex::build(
                self.memberships.as_ref(),
                self.programs.as_ref(),
                self.categories.as_ref(),
            )
            .await?;
            library
                .into_iter()
                .filter(|meditation| matches(query, &index.haystack_for(meditation)))
                .collect()
        };
        // Home-program decoration ships with the library list so the client
        // renders program links without a second request.
        let homes = self
            .memberships
            .home_programs()
            .await
            .map_err(ApplicationError::Repository)?;
        Ok(library
            .into_iter()
            .map(|meditation| {
                let home = homes.get(&meditation.id()).copied();
                LibraryEntry {
                    program_id: home.map(|(id, _)| id),
                    program_has_cover: home.map(|(_, has_cover)| has_cover).unwrap_or(false),
                    meditation,
                }
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "list_library_service.tests.rs"]
mod tests;
