use crate::ports::{media::BlobStore, SoundRepository};
use crate::ApplicationError;

pub(crate) async fn remove_sound(
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
