use std::sync::Arc;

use super::create_sound::{CreateSound, MAX_UPLOAD_BYTES};
use super::create_sound_handler::CreateSoundHandler;
use crate::ports::{
    media::{AudioValidator, BlobStore, ImageNormalizer, OpusConverter},
    CategoryRepository, NewSound, SoundRepository,
};
use crate::services::{
    ensure_category_exists, normalize_icon, parse_icon_key, remove_sound, store_sound_icon,
};
use crate::ApplicationError;
use chrono::Utc;
use soundscape_domain::{valid_name, Sound};

pub struct CreateSoundService {
    categories: Arc<dyn CategoryRepository>,
    sounds: Arc<dyn SoundRepository>,
    validator: Arc<dyn AudioValidator>,
    converter: Arc<dyn OpusConverter>,
    store: Arc<dyn BlobStore>,
    image_normalizer: Arc<dyn ImageNormalizer>,
}

impl CreateSoundService {
    pub fn new(
        categories: Arc<dyn CategoryRepository>,
        sounds: Arc<dyn SoundRepository>,
        validator: Arc<dyn AudioValidator>,
        converter: Arc<dyn OpusConverter>,
        store: Arc<dyn BlobStore>,
        image_normalizer: Arc<dyn ImageNormalizer>,
    ) -> Self {
        Self {
            categories,
            sounds,
            validator,
            converter,
            store,
            image_normalizer,
        }
    }

    async fn attach_icon_or_roll_back(
        &self,
        sound: Sound,
        icon: Vec<u8>,
    ) -> Result<Sound, ApplicationError> {
        match store_sound_icon(self.sounds.as_ref(), self.store.as_ref(), sound.id, icon).await {
            Ok(sound) => Ok(sound),
            Err(error) => {
                // Upload is all-or-nothing: do not leave a half-created sound.
                let _ = remove_sound(self.sounds.as_ref(), self.store.as_ref(), sound.id).await;
                Err(error)
            }
        }
    }
}

#[async_trait::async_trait]
impl CreateSoundHandler for CreateSoundService {
    async fn handle(&self, command: CreateSound) -> Result<Sound, ApplicationError> {
        let CreateSound {
            name,
            category_id,
            bytes,
            icon_key,
            icon,
        } = command;
        valid_name(&name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
        let icon_key = parse_icon_key(icon_key.as_deref())?;
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(ApplicationError::PayloadTooLarge);
        }
        ensure_category_exists(self.categories.as_ref(), category_id).await?;
        let icon = match icon {
            Some(bytes) => Some(normalize_icon(self.image_normalizer.as_ref(), bytes).await?),
            None => None,
        };
        let format = self
            .validator
            .sniff(&bytes)
            .await
            .map_err(|_| ApplicationError::UnsupportedMedia)?;
        let (stored_bytes, duration_ms, stored) = self
            .converter
            .convert(bytes, format)
            .await
            .map_err(ApplicationError::Storage)?;
        if duration_ms <= 0 {
            return Err(ApplicationError::UnsupportedMedia);
        }
        let key = self
            .store
            .put("soundscape", &name, stored.extension, &stored_bytes)
            .await
            .map_err(ApplicationError::Storage)?;
        let new_sound = NewSound {
            name: name.trim().into(),
            category_id,
            content_type: stored.content_type.into(),
            size_bytes: stored_bytes.len() as i64,
            storage_key: key.clone(),
            duration_ms,
            created_at: Utc::now(),
            icon_key,
        };
        let sound = match self.sounds.create(new_sound).await {
            Ok(sound) => sound,
            Err(error) => {
                let _ = self.store.delete(&key).await;
                return Err(ApplicationError::Repository(error));
            }
        };
        match icon {
            Some(icon) => self.attach_icon_or_roll_back(sound, icon).await,
            None => Ok(sound),
        }
    }
}

#[cfg(test)]
#[path = "create_sound_service.tests.rs"]
mod tests;
