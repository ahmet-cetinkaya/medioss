use kernel::{Actor, Role, UserId};

use crate::{ports::UserRepository, ApplicationError};

/// Sets the disabled flag of a user. Disabling oneself is always refused;
/// disabling the last active admin is refused to keep the instance
/// administrable.
pub async fn set_user_status(
    repository: &impl UserRepository,
    actor: &Actor,
    target: UserId,
    disabled: bool,
) -> Result<(), ApplicationError> {
    if actor.role() != Role::Admin {
        return Err(ApplicationError::Forbidden);
    }
    if actor.user_id() == target && disabled {
        return Err(ApplicationError::LastActiveAdmin);
    }
    let user = repository
        .find_by_id(target)
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;
    if disabled && user.role() == Role::Admin {
        let active_admins = repository
            .count_active_admins()
            .await
            .map_err(ApplicationError::from)?;
        if active_admins <= 1 {
            return Err(ApplicationError::LastActiveAdmin);
        }
    }
    repository
        .set_disabled(target, disabled)
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "set_user_status.tests.rs"]
mod tests;
