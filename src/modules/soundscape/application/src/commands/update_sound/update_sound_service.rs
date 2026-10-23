use std::sync::Arc;

use super::update_sound::UpdateSound;
use super::update_sound_handler::UpdateSoundHandler;
use crate::ports::{CategoryRepository, SoundRepository};
use crate::services::{ensure_category_exists, parse_icon_key};
use crate::ApplicationError;
use soundscape_domain::{valid_name, Sound};

pub struct UpdateSoundService {
    categories: Arc<dyn CategoryRepository>,
    sounds: Arc<dyn SoundRepository>,
}

impl UpdateSoundService {
    pub fn new(categories: Arc<dyn CategoryRepository>, sounds: Arc<dyn SoundRepository>) -> Self {
        Self { categories, sounds }
    }
}

#[async_trait::async_trait]
impl UpdateSoundHandler for UpdateSoundService {
    async fn handle(&self, command: UpdateSound) -> Result<Sound, ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
        let icon_key = parse_icon_key(command.icon_key.as_deref())?;
        ensure_category_exists(self.categories.as_ref(), command.category_id).await?;
        self.sounds
            .update(
                command.id,
                &command.name,
                command.category_id,
                icon_key.as_deref(),
            )
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)
    }
}
