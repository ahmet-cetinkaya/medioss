use std::sync::Arc;

use super::list_programs::ListPrograms;
use super::list_programs_handler::ListProgramsHandler;
use crate::ports::Program;
use crate::ports::ProgramRepository;
use crate::ApplicationError;

pub struct ListProgramsService {
    programs: Arc<dyn ProgramRepository>,
}

impl ListProgramsService {
    pub fn new(programs: Arc<dyn ProgramRepository>) -> Self {
        Self { programs }
    }
}

#[async_trait::async_trait]
impl ListProgramsHandler for ListProgramsService {
    async fn handle(&self, _query: ListPrograms) -> Result<Vec<Program>, ApplicationError> {
        // Lists every program (spec FR-025/031 views).
        self.programs
            .list_all()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_programs_service.tests.rs"]
mod tests;
