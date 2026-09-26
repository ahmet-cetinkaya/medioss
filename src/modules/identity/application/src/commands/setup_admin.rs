use identity_domain::{Email, RawPassword, User};

use crate::{
    ports::{Clock, InitialAdminRegistration, PasswordHasher, UserRepository},
    ApplicationError,
};

pub async fn setup_admin(
    repository: &impl UserRepository,
    hasher: &impl PasswordHasher,
    clock: &impl Clock,
    email: Email,
    password: &RawPassword,
) -> Result<User, ApplicationError> {
    let password_hash = hasher.hash(password).await?;
    let registration = InitialAdminRegistration::new(email, password_hash, clock.now());

    repository
        .insert_initial_admin_if_uninitialized(registration)
        .await
        .map_err(ApplicationError::from)
}

#[cfg(test)]
#[path = "setup_admin.tests.rs"]
mod tests;
