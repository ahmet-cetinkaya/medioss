use std::sync::Arc;

use super::list_programs_query::ListProgramsQuery;
use super::list_programs_query_validator::ListProgramsQueryValidator;
use crate::ports::Program;
use crate::ports::ProgramRepository;
use crate::ports::QueryHandler;
use crate::ApplicationError;

pub struct ListProgramsQueryHandler {
    programs: Arc<dyn ProgramRepository>,
    validator: ListProgramsQueryValidator,
}

impl ListProgramsQueryHandler {
    pub fn new(programs: Arc<dyn ProgramRepository>) -> Self {
        Self {
            programs,
            validator: ListProgramsQueryValidator,
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ListProgramsQuery> for ListProgramsQueryHandler {
    type Output = Vec<Program>;

    async fn handle(&self, query: ListProgramsQuery) -> Result<Self::Output, ApplicationError> {
        self.validator.validate(&query)?;
        // Lists every program (spec FR-025/031 views).
        self.programs
            .list_all()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_programs_query_handler.tests.rs"]
mod tests;
