use identity_domain::{Email, RawPassword, Role, User};
use kernel::Actor;

use crate::{
    ports::{Clock, MemberRegistration, PasswordHasher, UserRepository},
    ApplicationError,
};

pub async fn create_user(
    repository: &impl UserRepository,
    hasher: &impl PasswordHasher,
    clock: &impl Clock,
    actor: &Actor,
    email: Email,
    temporary_password: &RawPassword,
) -> Result<User, ApplicationError> {
    if actor.role() != Role::Admin {
        return Err(ApplicationError::Forbidden);
    }

    let password_hash = hasher.hash(temporary_password).await?;
    let registration = MemberRegistration::new(email, password_hash, clock.now());

    repository
        .insert_member(registration)
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "create_user.tests.rs"]
mod tests;
