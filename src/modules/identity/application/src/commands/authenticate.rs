use identity_domain::{Email, PasswordHash, User};

use crate::{
    ports::{Clock, PasswordHasher, UserRepository},
    ApplicationError,
};

pub struct Authenticate<'a, R, H, C> {
    repository: &'a R,
    hasher: &'a H,
    clock: &'a C,
    dummy_password_hash: PasswordHash,
}

impl<'a, R, H, C> Authenticate<'a, R, H, C>
where
    R: UserRepository,
    H: PasswordHasher,
    C: Clock,
{
    /// `dummy_password_hash` must be a fixed, valid production-parameter PHC.
    /// It keeps unknown and malformed identifiers on the same single-verify path
    /// as known users without generating a fresh expensive hash per request.
    pub fn new(
        repository: &'a R,
        hasher: &'a H,
        clock: &'a C,
        dummy_password_hash: PasswordHash,
    ) -> Self {
        Self {
            repository,
            hasher,
            clock,
            dummy_password_hash,
        }
    }

    pub async fn execute(
        &self,
        email_input: &str,
        password: &str,
    ) -> Result<User, ApplicationError> {
        let normalized_email = email_input.trim().to_lowercase();
        let user = match Email::parse(&normalized_email) {
            Ok(email) => self.repository.find_by_email(&email).await?,
            Err(_) => None,
        };
        let password_hash = user
            .as_ref()
            .map_or(&self.dummy_password_hash, User::password_hash);
        let is_password_valid = self.hasher.verify(password, password_hash).await?;
        let is_credential_active = user.as_ref().is_some_and(|known_user| {
            known_user
                .ensure_password_is_active_at(self.clock.now())
                .is_ok()
        });

        if is_password_valid && is_credential_active {
            return user.ok_or(ApplicationError::InvalidCredentials);
        }

        self.repository
            .increment_failed_attempt(&normalized_email, self.clock.now())
            .await?;
        Err(ApplicationError::InvalidCredentials)
    }
}

#[cfg(test)]
#[path = "authenticate.tests.rs"]
mod tests;
