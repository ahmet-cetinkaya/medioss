use std::sync::Arc;

use super::find_sound_icon::FindSoundIcon;
use super::find_sound_icon_handler::FindSoundIconHandler;
use crate::ports::SoundRepository;
use crate::ApplicationError;
use soundscape_domain::SoundIcon;

pub struct FindSoundIconService {
    sounds: Arc<dyn SoundRepository>,
}

impl FindSoundIconService {
    pub fn new(sounds: Arc<dyn SoundRepository>) -> Self {
        Self { sounds }
    }
}

#[async_trait::async_trait]
impl FindSoundIconHandler for FindSoundIconService {
    async fn handle(&self, query: FindSoundIcon) -> Result<SoundIcon, ApplicationError> {
        self.sounds
            .find(query.id)
            .await
            .map_err(ApplicationError::Repository)?
            .and_then(|sound| sound.icon)
            .ok_or(ApplicationError::NotFound)
    }
}
