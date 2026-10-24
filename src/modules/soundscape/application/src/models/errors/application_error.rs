use crate::ports::media::ImageNormalizationError;
use thiserror::Error;

const ICON_PROCESSING_FAILED: &str = "icon processing failed";
const UNIQUE_VIOLATION_MARKER: &str = "UNIQUE constraint failed";

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
    #[error("media processing is temporarily unavailable")]
    Busy,
    #[error("media storage operation failed: {0}")]
    Storage(String),
    #[error("repository operation failed: {0}")]
    Repository(String),
}

impl ApplicationError {
    /// Maps a category repository failure, turning a unique-name violation
    /// into `Duplicate`.
    pub fn from_category_repository(error: String) -> Self {
        if error.contains(UNIQUE_VIOLATION_MARKER) {
            Self::Duplicate
        } else {
            Self::Repository(error)
        }
    }
}

impl From<ImageNormalizationError> for ApplicationError {
    fn from(error: ImageNormalizationError) -> Self {
        match error {
            ImageNormalizationError::InvalidPayload => Self::UnsupportedMedia,
            ImageNormalizationError::OutputTooLarge => Self::PayloadTooLarge,
            ImageNormalizationError::WorkerFailed => {
                Self::Storage(ICON_PROCESSING_FAILED.to_owned())
            }
            ImageNormalizationError::Busy => Self::Busy,
        }
    }
}

#[cfg(test)]
#[path = "application_error.tests.rs"]
mod tests;
