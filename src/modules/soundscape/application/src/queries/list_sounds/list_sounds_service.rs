use std::sync::Arc;

use super::list_sounds::ListSounds;
use super::list_sounds_handler::ListSoundsHandler;
use crate::ports::SoundRepository;
use crate::ApplicationError;
use soundscape_domain::Sound;

pub struct ListSoundsService {
    sounds: Arc<dyn SoundRepository>,
}

impl ListSoundsService {
    pub fn new(sounds: Arc<dyn SoundRepository>) -> Self {
        Self { sounds }
    }
}

#[async_trait::async_trait]
impl ListSoundsHandler for ListSoundsService {
    async fn handle(&self, _query: ListSounds) -> Result<Vec<Sound>, ApplicationError> {
        self.sounds
            .list()
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "list_sounds_service.tests.rs"]
mod tests;
