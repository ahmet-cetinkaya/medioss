//! Creates a program with an optional validated cover image, stored in the
//! program's own folder.
use crate::commands::{validate_name, MAX_COVER_BYTES};
use crate::ports::{BlobStore, Clock, CoverValidator, Program, ProgramRepository};
use crate::ApplicationError;

/// Creates a program; cover bytes are sniffed, capped, and stored at
/// `{folder}/cover-{suffix}.{ext}` (spec FR-025/026).
pub async fn create_program<P, V, S, K>(
    programs: &P,
    cover_validator: &V,
    store: &S,
    clock: &K,
    title: &str,
    description: Option<&str>,
    cover_bytes: Option<Vec<u8>>,
) -> Result<Program, ApplicationError>
where
    P: ProgramRepository + ?Sized,
    V: CoverValidator + ?Sized,
    S: BlobStore + ?Sized,
    K: Clock + ?Sized,
{
    validate_name(title)?;
    let cover = match cover_bytes {
        Some(bytes) => {
            if bytes.len() > MAX_COVER_BYTES {
                return Err(ApplicationError::PayloadTooLarge);
            }
            let format = cover_validator
                .sniff(&bytes)
                .await
                .map_err(|_| ApplicationError::UnsupportedMedia)?;
            Some((bytes, format.extension()))
        }
        None => None,
    };
    let program = programs
        .insert(
            title.trim().to_owned(),
            description
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_owned),
            clock.now().await,
        )
        .await
        .map_err(ApplicationError::Repository)?;
    if let Some((bytes, ext)) = cover {
        let blob = match store.put(&program.folder, "cover", ext, &bytes).await {
            Ok(blob) => blob,
            Err(error) => {
                // The program row without a cover is partial state the
                // caller never asked for (they submitted title+cover
                // together) — undo it rather than leave a coverless program
                // and a 503.
                let _ = programs.delete(program.id).await;
                return Err(ApplicationError::Storage(error));
            }
        };
        if let Err(error) = programs.set_cover_key(program.id, &blob.storage_key).await {
            let _ = store.delete(&blob.storage_key).await;
            let _ = programs.delete(program.id).await;
            return Err(ApplicationError::Repository(error));
        }
    }
    programs
        .find_by_id(program.id)
        .await
        .map_err(ApplicationError::Repository)?
        .ok_or(ApplicationError::NotFound)
}
