use crate::{
    ports::{CategoryRepository, SoundRepository},
    ApplicationError,
};

pub async fn list_categories(
    repo: &dyn CategoryRepository,
) -> Result<Vec<soundscape_domain::SoundCategory>, ApplicationError> {
    repo.list().await.map_err(ApplicationError::Repository)
}
pub async fn list_sounds(
    repo: &dyn SoundRepository,
) -> Result<Vec<soundscape_domain::Sound>, ApplicationError> {
    repo.list().await.map_err(ApplicationError::Repository)
}
