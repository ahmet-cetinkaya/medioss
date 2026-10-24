use crate::ports::UserRepository;
use crate::ApplicationError;
use identity_domain::User;
use kernel::Role;

/// Shared refusal rule for account deletion and disabling: an active admin
/// must always remain. Lives here because admin deletion is the use case
/// that owns the rule; self-deletion reuses it.
pub(crate) async fn ensure_not_last_active_admin(
    repository: &dyn UserRepository,
    user: &User,
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
#[path = "last_active_admin_guard.tests.rs"]
mod tests;
