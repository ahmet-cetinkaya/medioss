//! Deletes a category; its programs become uncategorized.
use crate::ports::{CategoryRepository, ProgramRepository};
use crate::ApplicationError;

/// Deletes a category and sweeps it from every program (spec FR-032).
pub async fn delete_category<C, P>(
    categories: &C,
    programs: &P,
    category_id: i64,
) -> Result<(), ApplicationError>
where
    C: CategoryRepository,
    P: ProgramRepository,
{
    let deleted = categories
        .delete(category_id)
        .await
        .map_err(ApplicationError::Repository)?;
    if !deleted {
        return Err(ApplicationError::NotFound);
    }
    programs
        .clear_category(category_id)
        .await
        .map_err(ApplicationError::Repository)?;
    Ok(())
}
