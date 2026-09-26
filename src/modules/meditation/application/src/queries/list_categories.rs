//! Lists every category.
use crate::ports::{Category, CategoryRepository};
use crate::ApplicationError;

/// Lists every category (spec FR-030 views).
pub async fn list_categories<C>(categories: &C) -> Result<Vec<Category>, ApplicationError>
where
    C: CategoryRepository + ?Sized,
{
    categories
        .list_all()
        .await
        .map_err(ApplicationError::Repository)
}
