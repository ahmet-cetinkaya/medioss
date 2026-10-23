use crate::ports::MeditationRepository;
use crate::ApplicationError;

pub(crate) async fn ensure_meditation_exists(
    repository: &dyn MeditationRepository,
    meditation_id: i64,
) -> Result<(), ApplicationError> {
    let found = repository
        .find_by_id(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?;
    if found.is_some() {
        Ok(())
    } else {
        Err(ApplicationError::NotFound)
    }
}
