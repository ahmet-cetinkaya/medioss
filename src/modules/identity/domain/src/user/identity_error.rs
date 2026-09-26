use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum IdentityError {
    #[error("email is invalid")]
    InvalidEmail,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("user was not found")]
    UserNotFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("password must contain at least 8 characters")]
    WeakPassword,
    #[error("the actor is not allowed to perform this operation")]
    Forbidden,
    #[error("login is temporarily throttled; retry after {retry_after:?}")]
    LoginThrottled { retry_after: Duration },
    #[error("initial setup has already been completed")]
    SetupAlreadyCompleted,
    #[error("credential state is invalid")]
    InvalidCredentialState,
    #[error("login throttle state is invalid")]
    InvalidThrottleState,
    #[error("authentication version cannot be incremented")]
    AuthVersionOverflow,
}

impl IdentityError {
    pub fn retry_after(self) -> Option<Duration> {
        match self {
            Self::LoginThrottled { retry_after } => Some(retry_after),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "identity_error.tests.rs"]
mod tests;
