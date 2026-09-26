use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("password hashing operation failed")]
pub struct PasswordHasherError;
