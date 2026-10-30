use std::collections::HashMap;
use std::sync::Arc;

use super::library_entry::{LibraryEntry, LibraryProgram};
use super::list_library_query::ListLibraryQuery;
use super::list_library_query_validator::ListLibraryQueryValidator;
use super::matches::matches;
use super::search_index::SearchIndex;
use crate::ports::QueryHandler;
use crate::ports::{
    CategoryRepository, MeditationRepository, MembershipRepository, ProgramRepository,
};
use crate::ApplicationError;

pub struct ListLibraryQueryHandler {
    meditations: Arc<dyn MeditationRepository>,
    memberships: Arc<dyn MembershipRepository>,
    programs: Arc<dyn ProgramRepository>,
    categories: Arc<dyn CategoryRepository>,
    validator: ListLibraryQueryValidator,
}

impl ListLibraryQueryHandler {
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
            validator: ListLibraryQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListLibraryQuery> for ListLibraryQueryHandler {
    type Output = Vec<LibraryEntry>;

    async fn handle(&self, query: ListLibraryQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        let library = self
            .meditations
            .list_all()
            .await
            .map_err(ApplicationError::Repository)?;
        let library = if let Some(search) = self.validator.non_blank_search(&query) {
            let index = SearchIndex::build(
                self.memberships.as_ref(),
                self.programs.as_ref(),
                self.categories.as_ref(),
            )
            .await?;
            library
                .into_iter()
                .filter(|meditation| matches(search, &index.haystack_for(meditation)))
                .collect()
        } else {
            library
        };
        // Home-program decoration ships with the library list so the client
        // renders program links without a second request.
        let homes = self
            .memberships
            .home_programs()
            .await
            .map_err(ApplicationError::Repository)?;
        // Every program each meditation belongs to, for list cards that name
        // all memberships.
        let programs_by_meditation = self.programs_by_meditation().await?;
        Ok(library
            .into_iter()
            .map(|meditation| {
                let home = homes.get(&meditation.id()).copied();
                LibraryEntry {
                    program_id: home.map(|(id, _)| id),
                    program_has_cover: home.map(|(_, has_cover)| has_cover).unwrap_or(false),
                    programs: programs_by_meditation
                        .get(&meditation.id())
                        .cloned()
                        .unwrap_or_default(),
                    meditation,
                }
            })
            .collect())
    }
}

impl ListLibraryQueryHandler {
    /// Meditation id -> the programs it belongs to, earliest membership first.
    async fn programs_by_meditation(
        &self,
    ) -> Result<HashMap<i64, Vec<LibraryProgram>>, ApplicationError> {
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
                joined.push((
                    membership.added_at,
                    membership.meditation_id,
                    LibraryProgram {
                        id: program.id,
                        title: program.title.clone(),
                    },
                ));
            }
        }
        joined.sort_by_key(|(added_at, _, label)| (*added_at, label.id));
        let mut grouped: HashMap<i64, Vec<LibraryProgram>> = HashMap::new();
        for (_, meditation_id, label) in joined {
            grouped.entry(meditation_id).or_default().push(label);
        }
        Ok(grouped)
    }
}

#[cfg(test)]
#[path = "list_library_query_handler.tests.rs"]
mod tests;
