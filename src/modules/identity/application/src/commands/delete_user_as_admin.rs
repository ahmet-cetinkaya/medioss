use kernel::{Actor, Role, UserId};

use crate::{ports::UserRepository, ApplicationError};

/// Runs every guard for an admin-initiated deletion without mutating
/// anything. The composition root calls this before purging module
/// data, so a refused request never destroys the target's data.
pub async fn ensure_admin_can_delete_user(
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
    ensure_not_last_active_admin(repository, &user).await
}

/// Deletes a user as an admin. Deleting the last active admin is refused.
/// Callers must run [`ensure_admin_can_delete_user`] (and their purge
/// step) before this; the guards run again here so the command stays safe
/// when called directly.
pub async fn delete_user_as_admin(
    repository: &impl UserRepository,
    actor: &Actor,
    target: UserId,
) -> Result<(), ApplicationError> {
    ensure_admin_can_delete_user(repository, actor, target).await?;
    let user = repository
        .find_by_id(target)
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    repository
        .delete_login_attempts(&user.email().as_str().trim().to_lowercase())
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

pub(crate) async fn ensure_not_last_active_admin(
    repository: &impl UserRepository,
    user: &identity_domain::User,
) -> Result<(), ApplicationError> {
    if user.role() == Role::Admin && !user.is_disabled() {
        let active_admins = repository
            .count_active_admins()
            .await
            .map_err(ApplicationError::from)?;
        if active_admins <= 1 {
            return Err(ApplicationError::LastActiveAdmin);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "delete_user_as_admin.tests.rs"]
mod tests;
