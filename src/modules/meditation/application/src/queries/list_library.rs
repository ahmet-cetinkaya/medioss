//! Lists the shared library.
use crate::ports::MeditationRepository;
use crate::ApplicationError;

/// Lists every meditation, newest first (spec FR-007).
pub async fn list_library<R>(
    repository: &R,
) -> Result<Vec<meditation_domain::Meditation>, ApplicationError>
where
    R: MeditationRepository,
{
    repository
        .list_all()
        .await
        .map_err(ApplicationError::Repository)
}
