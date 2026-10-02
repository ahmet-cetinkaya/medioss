use identity_domain::IdentityError;
use thiserror::Error;

use crate::ports::{PasswordHasherError, RepositoryError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ApplicationError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("the actor is not allowed to perform this operation")]
    Forbidden,
    #[error("initial setup has already been completed")]
    SetupAlreadyCompleted,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("user was not found")]
    UserNotFound,
    #[error("application input is invalid: {0}")]
    InvalidInput(IdentityError),
    #[error("password hashing operation failed")]
    PasswordHasher,
    #[error("repository operation failed")]
    Repository,
    #[error("operation would leave the instance without an active admin")]
    LastActiveAdmin,
    #[error("request payload is too large")]
    PayloadTooLarge,
}

impl From<IdentityError> for ApplicationError {
    fn from(error: IdentityError) -> Self {
        Self::InvalidInput(error)
    }
}

impl From<PasswordHasherError> for ApplicationError {
    fn from(_: PasswordHasherError) -> Self {
        Self::PasswordHasher
    }
}

impl From<RepositoryError> for ApplicationError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::SetupAlreadyCompleted => Self::SetupAlreadyCompleted,
            RepositoryError::EmailAlreadyExists => Self::EmailAlreadyExists,
            RepositoryError::UserNotFound => Self::UserNotFound,
            RepositoryError::AuthVersionOverflow | RepositoryError::OperationFailed => {
                Self::Repository
            }
        }
    }
}
