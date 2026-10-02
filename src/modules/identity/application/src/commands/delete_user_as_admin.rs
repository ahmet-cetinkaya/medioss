use kernel::{Actor, Role, UserId};

use crate::{ports::UserRepository, ApplicationError};

/// Deletes a user as an admin. Deleting the last active admin is refused.
/// Cross-module data purge is orchestrated by the caller (web-api) before
/// the user row is removed.
pub async fn delete_user_as_admin(
    repository: &impl UserRepository,
    actor: &Actor,
    target: UserId,
) -> Result<(), ApplicationError> {
    if actor.role() != Role::Admin {
        return Err(ApplicationError::Forbidden);
    }
    let user = repository
        .find_by_id(target)
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    if user.role() == Role::Admin && !user.is_disabled() {
        let active_admins = repository
            .count_active_admins()
            .await
            .map_err(ApplicationError::from)?;
        if active_admins <= 1 {
            return Err(ApplicationError::LastActiveAdmin);
        }
    }
    repository
        .delete_login_attempts(user.email().as_str())
        .await
        .map_err(ApplicationError::from)?;
    let deleted = repository
        .delete_user(target)
        .await
        .map_err(ApplicationError::from)?;
    if !deleted {
        return Err(ApplicationError::UserNotFound);
    }
    Ok(())
}

#[cfg(test)]
#[path = "delete_user_as_admin.tests.rs"]
mod tests;
