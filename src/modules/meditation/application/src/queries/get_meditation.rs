//! Loads one meditation's metadata.
use crate::ports::MeditationRepository;
use crate::ApplicationError;

/// Loads one meditation (spec FR-008).
pub async fn get_meditation<R>(
    repository: &R,
    meditation_id: i64,
) -> Result<meditation_domain::Meditation, ApplicationError>
where
    R: MeditationRepository,
{
    repository
        .find_by_id(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)
}
