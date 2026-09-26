use crate::{ports::UserRepository, ApplicationError};

pub async fn count_users(repository: &impl UserRepository) -> Result<u64, ApplicationError> {
    repository.count().await.map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "count_users.tests.rs"]
mod tests;
