use crate::ports::SoundRepository;
use crate::ApplicationError;
use soundscape_domain::MixItem;

pub(crate) async fn ensure_sounds_exist(
    sounds: &dyn SoundRepository,
    items: &[MixItem],
) -> Result<(), ApplicationError> {
    for item in items {
        if sounds
            .find(item.sound_id)
            .await
            .map_err(ApplicationError::Repository)?
            .is_none()
        {
            return Err(ApplicationError::NotFound);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "ensure_sounds_exist.tests.rs"]
mod tests;
