//! Lists every program.
use crate::ports::{Program, ProgramRepository};
use crate::ApplicationError;

/// Lists every program (spec FR-025/031 views).
pub async fn list_programs<P>(programs: &P) -> Result<Vec<Program>, ApplicationError>
where
    P: ProgramRepository,
{
    programs
        .list_all()
        .await
        .map_err(ApplicationError::Repository)
}
