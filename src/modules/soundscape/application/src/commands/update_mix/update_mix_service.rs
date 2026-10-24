use std::sync::Arc;

use super::update_mix::UpdateMix;
use super::update_mix_handler::UpdateMixHandler;
use crate::ports::{Clock, MixRepository, NewMix, SoundRepository};
use crate::services::{ensure_sounds_exist, validate_mix_items};
use crate::ApplicationError;
use soundscape_domain::{valid_name, Mix};

pub struct UpdateMixService {
    mixes: Arc<dyn MixRepository>,
    sounds: Arc<dyn SoundRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateMixService {
    pub fn new(
        mixes: Arc<dyn MixRepository>,
        sounds: Arc<dyn SoundRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            mixes,
            sounds,
            clock,
        }
    }
}

#[async_trait::async_trait]
impl UpdateMixHandler for UpdateMixService {
    async fn handle(&self, command: UpdateMix) -> Result<Mix, ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("mix name"))?;
        validate_mix_items(&command.items)?;
        ensure_sounds_exist(self.sounds.as_ref(), &command.items).await?;
        self.mixes
            .replace(
                command.id,
                NewMix {
                    user_id: command.user_id,
                    name: command.name.trim().into(),
                    items: command.items,
                    now: self.clock.now().await,
                },
            )
            .await
            .map_err(ApplicationError::Repository)?
            .ok_or(ApplicationError::NotFound)
    }
}

#[cfg(test)]
#[path = "update_mix_service.tests.rs"]
mod tests;
