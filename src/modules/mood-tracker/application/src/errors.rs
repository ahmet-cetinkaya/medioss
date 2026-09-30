use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApplicationError {
    #[error("input is invalid: {0}")]
    InvalidInput(&'static str),
    #[error("the requested resource was not found")]
    NotFound,
    #[error("repository operation failed: {0}")]
    Repository(String),
}
