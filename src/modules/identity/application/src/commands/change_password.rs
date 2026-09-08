use identity_domain::{RawPassword, User};
use kernel::Actor;

use crate::{
    ports::{CredentialChange, PasswordHasher, UserRepository},
    ApplicationError,
};

pub async fn change_password(
    repository: &impl UserRepository,
    hasher: &impl PasswordHasher,
    actor: &Actor,
    new_password: &RawPassword,
) -> Result<User, ApplicationError> {
    let password_hash = hasher.hash(new_password).await?;
    let credential_change = CredentialChange::new(actor.user_id(), password_hash);

    repository
        .apply_credential_change(credential_change)
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "change_password.tests.rs"]
mod tests;
