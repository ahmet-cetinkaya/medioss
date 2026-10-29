use std::sync::Arc;

use super::create_sound_command::CreateSoundCommand;
use super::create_sound_command_validator::CreateSoundCommandValidator;
use crate::ports::{
    media::{AudioValidator, BlobStore, ImageNormalizer, OpusConverter, StoredAudio},
    staged_upload::{StagedMedia, UploadStaging},
    CategoryRepository, CommandHandler, NewSound, SoundRepository,
};
use crate::services::{
    ensure_category_exists, normalize_icon, parse_icon_key, remove_sound, store_sound_icon,
};
use crate::ApplicationError;
use chrono::Utc;
use soundscape_domain::Sound;

/// How much of the staged file the format sniffer gets to see. Mirrors meditation's
/// handler; the boundary rule forbids sharing it across modules.
const SNIFF_PREFIX_BYTES: usize = 64 * 1024;

pub struct CreateSoundCommandHandler {
    categories: Arc<dyn CategoryRepository>,
    sounds: Arc<dyn SoundRepository>,
    validator: Arc<dyn AudioValidator>,
    converter: Arc<dyn OpusConverter>,
    store: Arc<dyn BlobStore>,
    staging: Arc<dyn UploadStaging>,
    image_normalizer: Arc<dyn ImageNormalizer>,
    command_validator: CreateSoundCommandValidator,
}

impl CreateSoundCommandHandler {
    #[allow(clippy::too_many_arguments)] // every port plus the injected size cap
    pub fn new(
        categories: Arc<dyn CategoryRepository>,
        sounds: Arc<dyn SoundRepository>,
        validator: Arc<dyn AudioValidator>,
        converter: Arc<dyn OpusConverter>,
        store: Arc<dyn BlobStore>,
        staging: Arc<dyn UploadStaging>,
        image_normalizer: Arc<dyn ImageNormalizer>,
        max_upload_bytes: usize,
    ) -> Self {
        Self {
            categories,
            sounds,
            validator,
            converter,
            store,
            staging,
            image_normalizer,
            command_validator: CreateSoundCommandValidator::new(max_upload_bytes),
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
impl CommandHandler<CreateSoundCommand> for CreateSoundCommandHandler {
    type Output = Sound;

    async fn handle(&self, command: CreateSoundCommand) -> Result<Sound, ApplicationError> {
        let staged = command.media.clone();
        let result = self.create(command).await;
        if result.is_err() {
            // Idempotent: a file already moved or converted away is absent.
            let _ = self.staging.discard(&staged).await;
        }
        result
    }
}

impl CreateSoundCommandHandler {
    async fn create(&self, command: CreateSoundCommand) -> Result<Sound, ApplicationError> {
        self.command_validator.validate(&command)?;
        let CreateSoundCommand {
            name,
            category_id,
            media,
            icon_key,
            icon,
        } = command;
        let icon_key = parse_icon_key(icon_key.as_deref())?;
        ensure_category_exists(self.categories.as_ref(), category_id).await?;
        let icon = match icon {
            Some(bytes) => Some(normalize_icon(self.image_normalizer.as_ref(), bytes).await?),
            None => None,
        };
        let (stored_media, duration_ms, stored) = self.sniff_and_convert(media).await?;
        let key = match self
            .store
            .put_staged(stored_media.clone(), "soundscape", &name, stored.extension)
            .await
        {
            Ok(key) => key,
            Err(error) => {
                // The converted output may be a different file than the upload.
                let _ = self.staging.discard(&stored_media).await;
                return Err(ApplicationError::Storage(error));
            }
        };
        let new_sound = NewSound {
            name: name.trim().into(),
            category_id,
            content_type: stored.content_type.into(),
            size_bytes: stored_media.size_bytes as i64,
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

    async fn sniff_and_convert(
        &self,
        media: StagedMedia,
    ) -> Result<(StagedMedia, i64, StoredAudio), ApplicationError> {
        let prefix = self
            .staging
            .read_prefix(&media, SNIFF_PREFIX_BYTES)
            .await
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        let format = self
            .validator
            .sniff(&prefix)
            .await
            .map_err(|_| ApplicationError::UnsupportedMedia)?;
        let input = media.clone();
        let (converted, duration_ms, stored) = self
            .converter
            .convert(media, format)
            .await
            .map_err(ApplicationError::Storage)?;
        if converted.address != input.address {
            let _ = self.staging.discard(&input).await;
        }
        if duration_ms <= 0 {
            let _ = self.staging.discard(&converted).await;
            return Err(ApplicationError::UnsupportedMedia);
        }
        Ok((converted, duration_ms, stored))
    }
}

#[cfg(test)]
#[path = "create_sound_command_handler.tests.rs"]
mod tests;
