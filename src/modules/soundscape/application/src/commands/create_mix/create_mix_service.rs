use std::sync::Arc;

use super::create_mix::CreateMix;
use super::create_mix_handler::CreateMixHandler;
use crate::ports::{Clock, MixRepository, NewMix, SoundRepository};
use crate::services::{ensure_sounds_exist, validate_mix_items};
use crate::ApplicationError;
use soundscape_domain::{valid_name, Mix};

pub struct CreateMixService {
    mixes: Arc<dyn MixRepository>,
    sounds: Arc<dyn SoundRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateMixService {
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
impl CreateMixHandler for CreateMixService {
    async fn handle(&self, command: CreateMix) -> Result<Mix, ApplicationError> {
        valid_name(&command.name).map_err(|_| ApplicationError::InvalidInput("mix name"))?;
        validate_mix_items(&command.items)?;
        ensure_sounds_exist(self.sounds.as_ref(), &command.items).await?;
        self.mixes
            .create(NewMix {
                user_id: command.user_id,
                name: command.name.trim().into(),
                items: command.items,
                now: self.clock.now().await,
            })
            .await
            .map_err(ApplicationError::Repository)
    }
}

#[cfg(test)]
#[path = "create_mix_service.tests.rs"]
mod tests;
