use identity_domain::User;

use crate::{ports::UserRepository, ApplicationError};

pub async fn list_users(repository: &impl UserRepository) -> Result<Vec<User>, ApplicationError> {
    repository.list().await.map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "list_users.tests.rs"]
mod tests;
