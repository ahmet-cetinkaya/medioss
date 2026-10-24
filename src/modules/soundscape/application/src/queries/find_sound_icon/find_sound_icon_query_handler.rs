use std::sync::Arc;

use super::find_sound_icon_query::FindSoundIconQuery;
use super::find_sound_icon_query_validator::FindSoundIconQueryValidator;
use crate::ports::{QueryHandler, SoundRepository};
use crate::ApplicationError;
use soundscape_domain::SoundIcon;

pub struct FindSoundIconQueryHandler {
    sounds: Arc<dyn SoundRepository>,
    validator: FindSoundIconQueryValidator,
}

impl FindSoundIconQueryHandler {
    pub fn new(sounds: Arc<dyn SoundRepository>) -> Self {
        Self {
            sounds,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<FindSoundIconQuery> for FindSoundIconQueryHandler {
    type Output = SoundIcon;

    async fn handle(&self, query: FindSoundIconQuery) -> Result<SoundIcon, ApplicationError> {
        self.validator.validate(&query)?;
        self.sounds
            .find(query.id)
            .await
            .map_err(ApplicationError::Repository)?
            .and_then(|sound| sound.icon)
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "find_sound_icon_query_handler.tests.rs"]
mod tests;
