pub mod mixes;
pub mod sounds;

#[cfg(test)]
#[path = "commands.tests.rs"]
mod tests;

use crate::{
    ports::{CategoryDelete, CategoryRepository},
    ApplicationError,
};
use soundscape_domain::valid_name;

fn category_error(error: String) -> ApplicationError {
    if error.contains("UNIQUE constraint failed") {
        ApplicationError::Duplicate
    } else {
        ApplicationError::Repository(error)
    }
}

pub async fn create_category(
    repo: &dyn CategoryRepository,
    name: &str,
) -> Result<soundscape_domain::SoundCategory, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("category name"))?;
    repo.create(name).await.map_err(category_error)
}
pub async fn rename_category(
    repo: &dyn CategoryRepository,
    id: i64,
    name: &str,
) -> Result<soundscape_domain::SoundCategory, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("category name"))?;
    repo.rename(id, name)
        .await
        .map_err(category_error)?
        .ok_or(ApplicationError::NotFound)
}
pub async fn delete_category(
    repo: &dyn CategoryRepository,
    id: i64,
) -> Result<(), ApplicationError> {
    match repo
        .delete_empty(id)
        .await
        .map_err(ApplicationError::Repository)?
    {
        CategoryDelete::Deleted => Ok(()),
        CategoryDelete::NotFound => Err(ApplicationError::NotFound),
        CategoryDelete::InUse => Err(ApplicationError::CategoryInUse),
    }
}
