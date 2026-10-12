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
/// Location of a sound's uploaded icon image; `NotFound` when the sound or
/// its icon is absent.
pub async fn find_sound_icon(
    repo: &dyn SoundRepository,
    id: i64,
) -> Result<soundscape_domain::SoundIcon, ApplicationError> {
    repo.find(id)
        .await
        .map_err(ApplicationError::Repository)?
        .and_then(|sound| sound.icon)
        .ok_or(ApplicationError::NotFound)
}
