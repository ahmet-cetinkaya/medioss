use kernel::Actor;

use crate::{ports::UserRepository, ApplicationError};

/// Deletes the caller's own account. Refused when the caller is the last
/// active admin. Cross-module data purge is orchestrated by the caller
/// (web-api) before the user row is removed.
pub async fn delete_own_account(
    repository: &impl UserRepository,
    actor: &Actor,
) -> Result<(), ApplicationError> {
    let user = repository
        .find_by_id(actor.user_id())
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    if user.role() == kernel::Role::Admin && !user.is_disabled() {
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
        .delete_user(actor.user_id())
        .await
        .map_err(ApplicationError::from)?;
    if !deleted {
        return Err(ApplicationError::UserNotFound);
    }
    Ok(())
}

#[cfg(test)]
#[path = "delete_own_account.tests.rs"]
mod tests;
