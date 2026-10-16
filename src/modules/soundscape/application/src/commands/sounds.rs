use crate::{
    ports::{
        media::{AudioValidator, BlobStore, ImageNormalizer, OpusConverter},
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
    image_normalizer: &dyn ImageNormalizer,
    name: &str,
    category_id: i64,
    bytes: Vec<u8>,
    icon_key: Option<&str>,
    icon: Option<Vec<u8>>,
) -> Result<Sound, ApplicationError> {
    valid_name(name).map_err(|_| ApplicationError::InvalidInput("sound name"))?;
    let icon_key = parse_icon_key(icon_key)?;
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    ensure_category_exists(categories, category_id).await?;
    let icon = match icon {
        Some(bytes) => Some(normalize_icon(image_normalizer, bytes).await?),
        None => None,
    };
    let format = validator
        .sniff(&bytes)
        .await
        .map_err(|_| ApplicationError::UnsupportedMedia)?;
    let (stored_bytes, duration_ms, stored) = converter
        .convert(bytes, format)
        .await
        .map_err(ApplicationError::Storage)?;
    if duration_ms <= 0 {
        return Err(ApplicationError::UnsupportedMedia);
    }
    let key = store
        .put("soundscape", name, stored.extension, &stored_bytes)
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
    let sound = match sounds.create(new_sound).await {
        Ok(sound) => sound,
        Err(error) => {
            let _ = store.delete(&key).await;
            return Err(ApplicationError::Repository(error));
        }
    };
    match icon {
        Some(icon) => attach_icon_or_roll_back(sounds, store, sound, icon).await,
        None => Ok(sound),
    }
}

fn parse_icon_key(icon_key: Option<&str>) -> Result<Option<String>, ApplicationError> {
    Ok(icon_key
        .map(valid_icon_key)
        .transpose()
        .map_err(|_| ApplicationError::InvalidInput("icon key"))?
        .flatten())
}

async fn ensure_category_exists(
    categories: &dyn CategoryRepository,
    category_id: i64,
) -> Result<(), ApplicationError> {
    categories
        .find(category_id)
        .await
        .map_err(ApplicationError::Repository)?
        .map(|_| ())
        .ok_or(ApplicationError::NotFound)
}

async fn attach_icon_or_roll_back(
    sounds: &dyn SoundRepository,
    store: &dyn BlobStore,
    sound: Sound,
    icon: Vec<u8>,
) -> Result<Sound, ApplicationError> {
    match store_sound_icon(sounds, store, sound.id, icon).await {
        Ok(sound) => Ok(sound),
        Err(error) => {
            // Upload is all-or-nothing: do not leave a half-created sound.
            let _ = delete_sound(sounds, store, sound.id).await;
            Err(error)
        }
    }
}
async fn normalize_icon(
    normalizer: &dyn ImageNormalizer,
    bytes: Vec<u8>,
) -> Result<Vec<u8>, ApplicationError> {
    if bytes.len() > MAX_ICON_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    sniff_icon(&bytes).ok_or(ApplicationError::UnsupportedMedia)?;
    normalizer
        .normalize(bytes)
        .await
        .map_err(ApplicationError::from)
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
    let icon_key = parse_icon_key(icon_key)?;
    ensure_category_exists(categories, category_id).await?;
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
    image_normalizer: &dyn ImageNormalizer,
    id: i64,
    bytes: Vec<u8>,
) -> Result<Sound, ApplicationError> {
    let bytes = normalize_icon(image_normalizer, bytes).await?;
    store_sound_icon(sounds, store, id, bytes).await
}

async fn store_sound_icon(
    sounds: &dyn SoundRepository,
    store: &dyn BlobStore,
    id: i64,
    bytes: Vec<u8>,
) -> Result<Sound, ApplicationError> {
    let previous = sounds
        .find(id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?
        .icon;
    let key = store
        .put("soundscape", &format!("icon-{id}"), "webp", &bytes)
        .await
        .map_err(ApplicationError::Storage)?;
    let icon = SoundIcon {
        storage_key: key.clone(),
        content_type: "image/webp".into(),
    };
    match sounds.set_icon(id, Some(icon)).await {
        Ok(Some(sound)) => {
            if let Some(previous) = previous {
                let _ = store.delete(&previous.storage_key).await;
            }
            Ok(sound)
        }
        Ok(None) => {
            let _ = store.delete(&key).await;
            Err(ApplicationError::NotFound)
        }
        Err(error) => {
            // `set_icon` can fail after committing its update while hydrating
            // the returned sound. Keep the new blob so that update cannot
            // leave the database pointing at a deleted key.
            Err(ApplicationError::Repository(error))
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
