use std::sync::Arc;

use super::update_program::UpdateProgram;
use super::update_program_handler::UpdateProgramHandler;
use crate::ports::ProgramRepository;
use crate::services::validate_name;
use crate::ApplicationError;

pub struct UpdateProgramService {
    programs: Arc<dyn ProgramRepository>,
}

impl UpdateProgramService {
    pub fn new(programs: Arc<dyn ProgramRepository>) -> Self {
        Self { programs }
    }
}

#[async_trait::async_trait]
impl UpdateProgramHandler for UpdateProgramService {
    async fn handle(&self, command: UpdateProgram) -> Result<(), ApplicationError> {
        // Updates title and description (spec FR-025).
        let UpdateProgram {
            program_id,
            title,
            description,
        } = command;
        validate_name(&title)?;
        let updated = self
            .programs
            .update(
                program_id,
                title.trim().to_owned(),
                description
                    .as_deref()
                    .map(str::trim)
                    .filter(|text| !text.is_empty())
                    .map(str::to_owned),
            )
            .await
            .map_err(ApplicationError::Repository)?;
        if updated {
            Ok(())
        } else {
            Err(ApplicationError::NotFound)
        }
    }
}
