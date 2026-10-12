//! Renames a category; names are unique instance-wide.
use crate::ports::{CategoryRepository, Clock, RenameCategoryError};
use crate::ApplicationError;

/// Renames a category (admin content management; the Library groups and the
/// admin filter follow automatically).
pub async fn rename_category<C, K>(
    categories: &C,
    clock: &K,
    category_id: i64,
    name: &str,
) -> Result<(), ApplicationError>
where
    C: CategoryRepository + ?Sized,
    K: Clock + ?Sized,
{
    let name = name.trim();
    if name.is_empty() {
        return Err(ApplicationError::InvalidInput("name must not be empty"));
    }
    // A duplicate check against every other category, so renaming to the
    // same name (a no-op) stays allowed.
    let existing = categories
        .find_by_name(name)
        .await
        .map_err(ApplicationError::Repository)?;
    if existing.is_some_and(|other| other.id != category_id) {
        return Err(ApplicationError::DuplicateCategory);
    }
    let renamed = categories
        .rename(category_id, name.to_owned(), clock.now().await)
        .await
        .map_err(|error| match error {
            RenameCategoryError::Duplicate => ApplicationError::DuplicateCategory,
            RenameCategoryError::Repository(error) => ApplicationError::Repository(error),
        })?;
    if !renamed {
        return Err(ApplicationError::NotFound);
    }
    Ok(())
}
