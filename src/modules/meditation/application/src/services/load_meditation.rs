use crate::ports::MeditationRepository;
use crate::ApplicationError;
use meditation_domain::Meditation;

pub(crate) async fn load_meditation(
    repository: &dyn MeditationRepository,
    meditation_id: i64,
) -> Result<Meditation, ApplicationError> {
    repository
        .find_by_id(meditation_id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)
}
