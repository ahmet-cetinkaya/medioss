//! Sets or clears a meditation's description.
use crate::commands::validate_description;
use crate::ports::MeditationRepository;
use crate::ApplicationError;

/// Replaces the description; blank or `None` clears it. Returns the updated
/// meditation.
pub async fn update_meditation_description<R>(
    repository: &R,
    meditation_id: i64,
    description: Option<&str>,
) -> Result<meditation_domain::Meditation, ApplicationError>
where
    R: MeditationRepository + ?Sized,
{
    let description = validate_description(description)?;
    let meditation = repository
        .find_by_id(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?
        .with_description(description.as_deref())
        .map_err(|_| ApplicationError::InvalidDescription)?;
    let updated = repository
        .set_description(meditation_id, description.as_deref())
        .await
        .map_err(ApplicationError::Repository)?;
    if updated {
        Ok(meditation)
    } else {
        Err(ApplicationError::NotFound)
    }
}
