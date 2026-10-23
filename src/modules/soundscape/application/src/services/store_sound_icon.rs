use crate::ports::{media::BlobStore, SoundRepository};
use crate::ApplicationError;
use soundscape_domain::{Sound, SoundIcon};

const ICON_CONTENT_TYPE: &str = "image/webp";

pub(crate) async fn store_sound_icon(
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
        content_type: ICON_CONTENT_TYPE.into(),
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
