//! Creates a category; names are unique instance-wide.
use crate::ports::{Category, CategoryRepository, Clock};
use crate::ApplicationError;

/// Creates a category (spec FR-030).
pub async fn create_category<C, K>(
    categories: &C,
    clock: &K,
    name: &str,
) -> Result<Category, ApplicationError>
where
    C: CategoryRepository,
    K: Clock,
{
    let name = name.trim();
    if name.is_empty() {
        return Err(ApplicationError::InvalidInput("name must not be empty"));
    }
    if categories
        .find_by_name(name)
        .await
        .map_err(ApplicationError::Repository)?
        .is_some()
    {
        return Err(ApplicationError::DuplicateCategory);
    }
    categories
        .insert(name.to_owned(), clock.now().await)
        .await
        .map_err(ApplicationError::Repository)
}
