//! Uploads audio: sniffs, converts to Opus, stores, persists the row.
use crate::commands::{validate_name, MAX_UPLOAD_BYTES};
use crate::ports::{AudioValidator, BlobStore, Clock, MeditationRepository, OpusConverter};
use crate::ApplicationError;

/// Uploads meditation audio. Garbage bytes, oversized payloads, and
/// conversion failures leave nothing stored (spec FR-001..004).
pub async fn upload_meditation<R, V, C, S, K>(
    repository: &R,
    validator: &V,
    converter: &C,
    store: &S,
    clock: &K,
    name: &str,
    bytes: Vec<u8>,
) -> Result<meditation_domain::Meditation, ApplicationError>
where
    R: MeditationRepository,
    V: AudioValidator,
    C: OpusConverter,
    S: BlobStore,
    K: Clock,
{
    validate_name(name)?;
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    let format = validator
        .sniff(&bytes)
        .await
        .map_err(|_| ApplicationError::UnsupportedMedia)?;
    let stored_bytes = converter
        .convert(bytes, format)
        .await
        .map_err(|_| ApplicationError::UnsupportedMedia)?;
    let blob = store
        .put(&stored_bytes)
        .await
        .map_err(ApplicationError::Storage)?;
    match repository
        .insert(
            name.trim().to_owned(),
            meditation_domain::STORED_CONTENT_TYPE,
            blob.size_bytes,
            blob.storage_key.clone(),
            clock.now().await,
        )
        .await
    {
        Ok(meditation) => Ok(meditation),
        Err(error) => {
            // The row is the source of truth: with no row, the blob must not
            // survive either. Best effort — the insert error is primary.
            let _ = store.delete(&blob.storage_key).await;
            Err(ApplicationError::Repository(error))
        }
    }
}
