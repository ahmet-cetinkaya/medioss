use kernel::Actor;

use crate::{
    commands::delete_user_as_admin::ensure_not_last_active_admin, ports::UserRepository,
    ApplicationError,
};

/// Runs every guard for a self-deletion without mutating anything. The
/// composition root calls this before purging module data, so a refused
/// request (last active admin) never destroys the caller's data.
pub async fn ensure_can_delete_own_account(
    repository: &impl UserRepository,
    actor: &Actor,
) -> Result<(), ApplicationError> {
    let user = repository
        .find_by_id(actor.user_id())
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    ensure_not_last_active_admin(repository, &user).await
}

/// Deletes the caller's own account. Refused when the caller is the last
/// active admin. Callers must run [`ensure_can_delete_own_account`] (and
/// their purge step) before this; the guard runs again here so the
/// command stays safe when called directly.
pub async fn delete_own_account(
    repository: &impl UserRepository,
    actor: &Actor,
) -> Result<(), ApplicationError> {
    ensure_can_delete_own_account(repository, actor).await?;
    let user = repository
        .find_by_id(actor.user_id())
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    repository
        .delete_login_attempts(&user.email().as_str().trim().to_lowercase())
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
