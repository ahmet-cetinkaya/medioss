use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApplicationError {
    #[error("input is invalid: {0}")]
    InvalidInput(&'static str),
    #[error("the requested resource was not found")]
    NotFound,
    #[error("a resource with that name already exists")]
    Duplicate,
    #[error("the category still contains sounds")]
    CategoryInUse,
    #[error("the uploaded bytes are not supported audio")]
    UnsupportedMedia,
    #[error("the payload exceeds the allowed size")]
    PayloadTooLarge,
    #[error("media storage operation failed: {0}")]
    Storage(String),
    #[error("repository operation failed: {0}")]
    Repository(String),
}
