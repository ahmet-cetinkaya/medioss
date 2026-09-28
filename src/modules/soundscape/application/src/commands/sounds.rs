use crate::{
    ports::{
        media::{AudioValidator, BlobStore, OpusConverter},
        CategoryRepository, NewSound, SoundRepository,
    },
    ApplicationError,
};
use chrono::Utc;
use soundscape_domain::{valid_name, Sound};
pub const MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;
#[allow(clippy::too_many_arguments)] // ports plus media payload are naturally flat
pub async fn create_sound(
    categories: &dyn CategoryRepository,
    sounds: &dyn SoundRepository,
    validator: &dyn AudioValidator,
    converter: &dyn OpusConverter,
    store: &dyn BlobStore,
    name: &str,
    category_id: i64,
    bytes: Vec<u8>,
) -> Result<Sound, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    if categories
        .find(category_id)
        .await
        .map_err(ApplicationError::Repository)?
        .is_none()
    {
        return Err(ApplicationError::NotFound);
    }
    let format = validator
        .sniff(&bytes)
        .await
        .map_err(|_| ApplicationError::UnsupportedMedia)?;
    let (stored_bytes, duration, stored) = converter
        .convert(bytes, format)
        .await
        .map_err(ApplicationError::Storage)?;
    if duration <= 0 {
        return Err(ApplicationError::UnsupportedMedia);
    }
    let key = store
        .put("soundscape", name, stored.extension, &stored_bytes)
        .await
        .map_err(ApplicationError::Storage)?;
    let result = sounds
        .create(NewSound {
            name: name.trim().into(),
            category_id,
            content_type: stored.content_type.into(),
            size_bytes: stored_bytes.len() as i64,
            storage_key: key.clone(),
            duration_ms: duration,
            created_at: Utc::now(),
        })
        .await
        .map_err(ApplicationError::Repository);
    if result.is_err() {
        let _ = store.delete(&key).await;
    }
    result
}
pub async fn delete_sound(
    sounds: &dyn SoundRepository,
    store: &dyn BlobStore,
    id: i64,
) -> Result<(), ApplicationError> {
    let sound = sounds
        .delete_with_mix_items(id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?;
    store
        .delete(&sound.storage_key)
        .await
        .map_err(ApplicationError::Storage)
}
