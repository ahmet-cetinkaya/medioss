use crate::ports::CoverNormalizationError;
use thiserror::Error;

/// Why a meditation use-case failed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApplicationError {
    #[error("input is invalid: {0}")]
    InvalidInput(&'static str),
    #[error("description must be at most 2000 characters")]
    InvalidDescription,
    #[error("meditation_ids must contain exactly the program's meditations")]
    InvalidProgramOrder,
    #[error("the uploaded bytes are not a supported media format")]
    UnsupportedMedia,
    #[error("the payload exceeds the allowed size")]
    PayloadTooLarge,
    #[error("media processing is temporarily unavailable")]
    Busy,
    #[error("the requested resource was not found")]
    NotFound,
    #[error("a category with that name already exists")]
    DuplicateCategory,
    #[error("media storage operation failed: {0}")]
    Storage(String),
    #[error("repository operation failed: {0}")]
    Repository(String),
}

const COVER_PROCESSING_FAILED: &str = "cover processing failed";

impl From<CoverNormalizationError> for ApplicationError {
    fn from(error: CoverNormalizationError) -> Self {
        match error {
            CoverNormalizationError::InvalidPayload => Self::UnsupportedMedia,
            CoverNormalizationError::OutputTooLarge => Self::PayloadTooLarge,
            CoverNormalizationError::WorkerFailed => {
                Self::Storage(COVER_PROCESSING_FAILED.to_owned())
            }
            CoverNormalizationError::Busy => Self::Busy,
        }
    }
}

#[cfg(test)]
#[path = "application_error.tests.rs"]
mod tests;
