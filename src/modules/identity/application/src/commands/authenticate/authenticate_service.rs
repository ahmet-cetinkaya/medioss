use std::sync::Arc;
use std::time::Duration;

use super::authenticate::Authenticate;
use super::authenticate_handler::AuthenticateHandler;
use crate::ports::{
    Clock, LoginAdmission, LoginThrottleRepository, PasswordHasher, UserRepository,
};
use crate::ApplicationError;
use identity_domain::{Email, IdentityError, PasswordHash, User};

pub struct AuthenticateService {
    // `dummy_password_hash` must be a fixed, valid production-parameter PHC.
    // It keeps unknown and malformed identifiers on the same single-verify
    // path as known users without generating a fresh expensive hash per
    // request.
    repository: Arc<dyn UserRepository>,
    throttle: Arc<dyn LoginThrottleRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
    admission: Arc<dyn LoginAdmission>,
    dummy_password_hash: PasswordHash,
}

fn login_throttled(retry_after: Duration) -> ApplicationError {
    IdentityError::LoginThrottled { retry_after }.into()
}

impl AuthenticateService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repository: Arc<dyn UserRepository>,
        throttle: Arc<dyn LoginThrottleRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
        admission: Arc<dyn LoginAdmission>,
        dummy_password_hash: PasswordHash,
    ) -> Self {
        Self {
            repository,
            throttle,
            hasher,
            clock,
            admission,
            dummy_password_hash,
        }
    }
}

#[async_trait::async_trait]
impl AuthenticateHandler for AuthenticateService {
    async fn handle(&self, command: Authenticate) -> Result<User, ApplicationError> {
        if command.bypass_throttle {
            return self
                .verify_credentials(&command.identifier, &command.password)
                .await;
        }
        self.execute_with_throttle(&command.identifier, &command.password)
            .await
    }
}

impl AuthenticateService {
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
            if user.as_ref().is_some_and(User::is_disabled) {
                // A disabled account must look exactly like unknown
                // credentials: no oracle about the account's state.
                self.throttle
                    .record_failed_attempt(&normalized_identifier, attempted_at)
                    .await?;
                return Err(ApplicationError::InvalidCredentials);
            }
            self.throttle
                .clear_failed_attempts(&normalized_identifier)
                .await?;
            return user.ok_or(ApplicationError::InvalidCredentials);
        }

        self.throttle
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
            .throttle
            .failed_attempts(normalized_identifier)
            .await?
            .and_then(|attempts| attempts.retry_after(now));

        match retry_after {
            Some(delay) => Err(login_throttled(delay)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "authenticate_service.tests.rs"]
mod tests;
