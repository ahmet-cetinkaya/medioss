use std::time::Duration;

use identity_domain::{Email, IdentityError, PasswordHash, User};

use crate::{
    ports::{Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository},
    ApplicationError,
};

pub struct Authenticate<'a, R, H, C, A> {
    repository: &'a R,
    hasher: &'a H,
    clock: &'a C,
    admission: &'a A,
    dummy_password_hash: PasswordHash,
}

impl<'a, R, H, C, A> Authenticate<'a, R, H, C, A>
where
    R: UserRepository + LoginThrottleRepository,
    H: PasswordHasher,
    C: Clock,
    A: LoginAdmission,
{
    /// `dummy_password_hash` must be a fixed, valid production-parameter PHC.
    /// It keeps unknown and malformed identifiers on the same single-verify path
    /// as known users without generating a fresh expensive hash per request.
    pub fn new(
        repository: &'a R,
        hasher: &'a H,
        clock: &'a C,
        admission: &'a A,
        dummy_password_hash: PasswordHash,
    ) -> Self {
        Self {
            repository,
            hasher,
            clock,
            admission,
            dummy_password_hash,
        }
    }

    pub async fn execute(
        &self,
        email_input: &str,
        password: &str,
    ) -> Result<User, ApplicationError> {
        self.execute_with_throttle(email_input, password).await
    }

    pub async fn execute_bypassing_throttle(
        &self,
        email_input: &str,
        password: &str,
    ) -> Result<User, ApplicationError> {
        self.verify_credentials(email_input, password).await
    }

    async fn execute_with_throttle(
        &self,
        email_input: &str,
        password: &str,
    ) -> Result<User, ApplicationError> {
        let normalized_identifier = email_input.trim().to_lowercase();
        let attempted_at = self.clock.now();
        self.ensure_identifier_is_ready(&normalized_identifier, attempted_at)
            .await?;
        let permit = self.admission.try_acquire().map_err(login_throttled)?;
        let result = self.verify_credentials(email_input, password).await;
        drop(permit);
        result
    }

    async fn verify_credentials(
        &self,
        email_input: &str,
        password: &str,
    ) -> Result<User, ApplicationError> {
        let normalized_identifier = email_input.trim().to_lowercase();
        let attempted_at = self.clock.now();
        let user = match Email::parse(&normalized_identifier) {
            Ok(email) => self.repository.find_by_email(&email).await?,
            Err(_) => None,
        };
        let password_hash = user
            .as_ref()
            .map_or(&self.dummy_password_hash, User::password_hash);
        let is_password_valid = self.hasher.verify(password, password_hash).await?;
        let is_credential_active = user.as_ref().is_some_and(|known_user| {
            known_user
                .ensure_password_is_active_at(attempted_at)
                .is_ok()
        });

        if is_password_valid && is_credential_active {
            self.repository
                .clear_failed_attempts(&normalized_identifier)
                .await?;
            return user.ok_or(ApplicationError::InvalidCredentials);
        }

        self.repository
            .record_failed_attempt(&normalized_identifier, attempted_at)
            .await?;
        Err(ApplicationError::InvalidCredentials)
    }

    async fn ensure_identifier_is_ready(
        &self,
        normalized_identifier: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ApplicationError> {
        let retry_after = self
            .repository
            .failed_attempts(normalized_identifier)
            .await?
            .and_then(|attempts| attempts.retry_after(now));

        match retry_after {
            Some(delay) => Err(login_throttled(delay)),
            None => Ok(()),
        }
    }
}

fn login_throttled(retry_after: Duration) -> ApplicationError {
    IdentityError::LoginThrottled { retry_after }.into()
}

#[cfg(test)]
#[path = "authenticate.tests.rs"]
mod tests;
