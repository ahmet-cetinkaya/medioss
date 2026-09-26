use crate::{ports::UserRepository, ApplicationError};
use identity_domain::User;
use kernel::UserId;

pub async fn find_user_by_id(
    repository: &impl UserRepository,
    id: UserId,
) -> Result<Option<User>, ApplicationError> {
    repository
        .find_by_id(id)
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "find_user_by_id.tests.rs"]
mod tests;
