use std::sync::Arc;

use super::list_library::ListLibrary;
use super::list_library_handler::ListLibraryHandler;
use crate::models::SearchIndex;
use crate::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::services::matches;
use crate::ApplicationError;
use meditation_domain::Meditation;

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
    async fn handle(&self, query: ListLibrary) -> Result<Vec<Meditation>, ApplicationError> {
        let library = self
            .meditations
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        let query = query.query.as_deref();
        let is_blank = query.map(str::trim).is_none_or(str::is_empty);
        if is_blank {
            return Ok(library);
        }
        let query = query.unwrap_or_default();
        let index = SearchIndex::build(
            self.memberships.as_ref(),
            self.programs.as_ref(),
            self.categories.as_ref(),
        )
        .await?;
        Ok(library
            .into_iter()
            .filter(|meditation| matches(query, &index.haystack_for(meditation)))
            .collect())
    }
}

#[cfg(test)]
#[path = "list_library_service.tests.rs"]
mod tests;
