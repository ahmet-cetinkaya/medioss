//! Assigns or clears a program's category.
use crate::ports::{CategoryRepository, ProgramRepository};
use crate::ApplicationError;

/// Assigns a category to a program, or clears it with `None` (spec FR-031).
pub async fn set_program_category<P, C>(
    programs: &P,
    categories: &C,
    program_id: i64,
    category_id: Option<i64>,
) -> Result<(), ApplicationError>
where
    P: ProgramRepository,
    C: CategoryRepository,
{
    if let Some(category_id) = category_id {
        categories
            .find_by_id(category_id)
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)?;
    }
    let set = programs
        .set_category(program_id, category_id)
        .await
        .map_err(ApplicationError::Repository)?;
    if set {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}
