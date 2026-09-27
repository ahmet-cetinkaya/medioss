//! Uploads audio into a program folder: sniffs, converts to Opus, stores,
//! persists the row, and records the first program membership (spec FR-001).
use crate::commands::{validate_name, MAX_UPLOAD_BYTES};
use crate::ports::{
    AudioValidator, BlobStore, Clock, ConvertError, MeditationRepository, MembershipRepository,
    OpusConverter, ProgramRepository,
};
use crate::ApplicationError;

fn map_convert_error(error: ConvertError) -> ApplicationError {
    match error {
        ConvertError::Decode => ApplicationError::UnsupportedMedia,
        ConvertError::Unavailable => {
            ApplicationError::Storage("the Opus encoder is unavailable".to_owned())
        }
    }
}

/// Uploads meditation audio belonging to `program_id` (the first, mandatory
/// program). Garbage bytes, oversized payloads, and conversion failures
/// leave nothing stored (spec FR-001..004).
#[allow(clippy::too_many_arguments)]
pub async fn upload_meditation<R, V, C, S, K, M, P>(
    repository: &R,
    memberships: &M,
    programs: &P,
    validator: &V,
    converter: &C,
    store: &S,
    clock: &K,
    name: &str,
    program_id: i64,
    bytes: Vec<u8>,
) -> Result<meditation_domain::Meditation, ApplicationError>
where
    R: MeditationRepository + ?Sized,
    V: AudioValidator + ?Sized,
    C: OpusConverter + ?Sized,
    S: BlobStore + ?Sized,
    K: Clock + ?Sized,
    M: MembershipRepository + ?Sized,
    P: ProgramRepository + ?Sized,
{
    validate_name(name)?;
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(ApplicationError::PayloadTooLarge);
    }
    let program = programs
        .find_by_id(program_id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)?;
    let format = validator
        .sniff(&bytes)
        .await
        .map_err(|_| ApplicationError::UnsupportedMedia)?;
    let stored_bytes = converter
        .convert(bytes, format)
        .await
        .map_err(map_convert_error)?;
    let blob = store
        .put(&program.folder, name, "opus", &stored_bytes)
        .await
        .map_err(ApplicationError::Storage)?;
    let meditation = match repository
        .insert(
            name.trim().to_owned(),
            meditation_domain::STORED_CONTENT_TYPE,
            blob.size_bytes,
            blob.storage_key.clone(),
            clock.now().await,
        )
        .await
    {
        Ok(meditation) => meditation,
        Err(error) => {
            // The row is the source of truth: with no row, the blob must not
            // survive either. Best effort — the insert error is primary.
            let _ = store.delete(&blob.storage_key).await;
            return Err(ApplicationError::Repository(error));
        }
    };
    memberships
        .add(program_id, meditation.id(), clock.now().await)
        .await;
    Ok(meditation)
}
