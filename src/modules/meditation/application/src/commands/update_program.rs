//! Updates a program's title and description.
use crate::commands::validate_name;
use crate::ports::ProgramRepository;
use crate::ApplicationError;

/// Updates title and description (spec FR-025).
pub async fn update_program<P>(
    programs: &P,
    program_id: i64,
    title: &str,
    description: Option<&str>,
) -> Result<(), ApplicationError>
where
    P: ProgramRepository + ?Sized,
{
    validate_name(title)?;
    let updated = programs
        .update(
            program_id,
            title.trim().to_owned(),
            description
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
