//! Converts the web-api image normalizer's outcomes into each module's port
//! error. Modules cannot share code, so the three conversions live here.

use crate::image_normalization::ImageNormalizationError;

impl From<ImageNormalizationError> for identity_application::ports::AvatarStoreError {
    fn from(error: ImageNormalizationError) -> Self {
        match error {
            ImageNormalizationError::InvalidPayload => Self::InvalidPayload,
            ImageNormalizationError::OutputTooLarge => Self::OutputTooLarge,
            ImageNormalizationError::WorkerFailed => Self::WorkerFailed,
            ImageNormalizationError::Busy => Self::Busy,
        }
    }
}

impl From<ImageNormalizationError> for meditation_application::ports::CoverNormalizationError {
    fn from(error: ImageNormalizationError) -> Self {
        match error {
            ImageNormalizationError::InvalidPayload => Self::InvalidPayload,
            ImageNormalizationError::OutputTooLarge => Self::OutputTooLarge,
            ImageNormalizationError::WorkerFailed => Self::WorkerFailed,
            ImageNormalizationError::Busy => Self::Busy,
        }
    }
}

impl From<ImageNormalizationError>
    for soundscape_application::ports::media::ImageNormalizationError
{
    fn from(error: ImageNormalizationError) -> Self {
        match error {
            ImageNormalizationError::InvalidPayload => Self::InvalidPayload,
            ImageNormalizationError::OutputTooLarge => Self::OutputTooLarge,
            ImageNormalizationError::WorkerFailed => Self::WorkerFailed,
            ImageNormalizationError::Busy => Self::Busy,
        }
    }
}

#[cfg(test)]
#[path = "normalization_adapters.tests.rs"]
mod tests;
