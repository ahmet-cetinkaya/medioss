use identity_domain::DisplayName;
use kernel::Actor;

use crate::{ports::UserRepository, ApplicationError};

pub async fn set_display_name(
    repository: &impl UserRepository,
    actor: &Actor,
    display_name: Option<DisplayName>,
) -> Result<identity_domain::User, ApplicationError> {
    let user = repository
        .find_by_id(actor.user_id())
        .await
        .map_err(ApplicationError::from)?
        .ok_or(ApplicationError::UserNotFound)?;

    repository
        .set_profile(actor.user_id(), display_name, user.avatar())
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "set_display_name.tests.rs"]
mod tests;
