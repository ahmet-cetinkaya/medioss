use crate::{
    ports::{
        media::{AudioValidator, BlobStore, OpusConverter},
        CategoryRepository, NewSound, SoundRepository,
    },
    ApplicationError,
};
use chrono::Utc;
use soundscape_domain::{sniff_icon, valid_icon_key, valid_name, Sound, SoundIcon, MAX_ICON_BYTES};
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
    icon_key: Option<&str>,
    icon: Option<Vec<u8>>,
) -> Result<Sound, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
    let icon_key = icon_key
        .map(valid_icon_key)
        .transpose()
        .map_err(|_| ApplicationError::InvalidInput("icon key"))?
        .flatten();
    if let Some(icon) = &icon {
        check_icon(icon)?;
    }
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
            icon_key,
        })
        .await
        .map_err(ApplicationError::Repository);
    let sound = match result {
        Ok(sound) => sound,
        Err(error) => {
            let _ = store.delete(&key).await;
            return Err(error);
        }
    };
    match icon {
        Some(icon) => match set_sound_icon(sounds, store, sound.id, icon).await {
            Ok(sound) => Ok(sound),
            Err(error) => {
                // Upload is all-or-nothing: do not leave a half-created sound.
                let _ = delete_sound(sounds, store, sound.id).await;
                Err(error)
            }
        },
        None => Ok(sound),
    }
}
fn check_icon(bytes: &[u8]) -> Result<(&'static str, &'static str), ApplicationError> {
    if bytes.len() > MAX_ICON_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    sniff_icon(bytes).ok_or(ApplicationError::UnsupportedMedia)
}
/// Edits name, category and bundled-glyph key. `icon_key` replaces the stored
/// value; `None` or blank clears it.
pub async fn update_sound(
    categories: &dyn CategoryRepository,
    sounds: &dyn SoundRepository,
    id: i64,
    name: &str,
    category_id: i64,
    icon_key: Option<&str>,
) -> Result<Sound, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
    let icon_key = icon_key
        .map(valid_icon_key)
        .transpose()
        .map_err(|_| ApplicationError::InvalidInput("icon key"))?
        .flatten();
    if categories
        .find(category_id)
        .await
        .map_err(ApplicationError::Repository)?
        .is_none()
    {
        return Err(ApplicationError::NotFound);
    }
    sounds
        .update(id, name, category_id, icon_key.as_deref())
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)
}
/// Stores an uploaded icon image (sniffed png/jpeg/webp, <= 1 MB), replacing
/// any previous one.
pub async fn set_sound_icon(
    sounds: &dyn SoundRepository,
    store: &dyn BlobStore,
    id: i64,
    bytes: Vec<u8>,
) -> Result<Sound, ApplicationError> {
    let (content_type, ext) = check_icon(&bytes)?;
    let previous = sounds
        .find(id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?
        .icon;
    let key = store
        .put("soundscape", &format!("icon-{id}"), ext, &bytes)
        .await
        .map_err(ApplicationError::Storage)?;
    let icon = SoundIcon {
        storage_key: key.clone(),
        content_type: content_type.into(),
    };
    match sounds.set_icon(id, Some(icon)).await {
        Ok(Some(sound)) => {
            if let Some(previous) = previous {
                let _ = store.delete(&previous.storage_key).await;
            }
            Ok(sound)
        }
        other => {
            let _ = store.delete(&key).await;
            match other {
                Ok(_) => Err(ApplicationError::NotFound),
                Err(error) => Err(ApplicationError::Repository(error)),
            }
        }
    }
}
/// Removes the uploaded icon image (the bundled `icon_key` is untouched).
pub async fn clear_sound_icon(
    sounds: &dyn SoundRepository,
    store: &dyn BlobStore,
    id: i64,
) -> Result<Sound, ApplicationError> {
    let previous = sounds
        .find(id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?
        .icon;
    let sound = sounds
        .set_icon(id, None)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?;
    if let Some(previous) = previous {
        let _ = store.delete(&previous.storage_key).await;
    }
    Ok(sound)
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
    if let Some(icon) = &sound.icon {
        let _ = store.delete(&icon.storage_key).await;
    }
    store
        .delete(&sound.storage_key)
        .await
        .map_err(ApplicationError::Storage)
}
