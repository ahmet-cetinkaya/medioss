use crate::{ports::UserRepository, queries::count_users, ApplicationError};

pub async fn is_setup_required(repository: &impl UserRepository) -> Result<bool, ApplicationError> {
    Ok(count_users(repository).await? == 0)
}

#[cfg(test)]
#[path = "setup_status.tests.rs"]
mod tests;
