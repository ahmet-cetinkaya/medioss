use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RepositoryError {
    #[error("initial setup has already been completed")]
    SetupAlreadyCompleted,
    #[error("email already exists")]
    EmailAlreadyExists,
    #[error("user was not found")]
    UserNotFound,
    #[error("authentication version cannot be incremented")]
    AuthVersionOverflow,
    #[error("repository operation failed")]
    OperationFailed,
}
