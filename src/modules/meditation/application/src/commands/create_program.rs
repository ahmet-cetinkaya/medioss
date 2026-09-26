//! Creates a program with an optional validated cover image.
use crate::commands::{validate_name, MAX_COVER_BYTES};
use crate::ports::{BlobStore, Clock, CoverValidator, Program, ProgramRepository};
use crate::ApplicationError;

/// Creates a program; cover bytes are sniffed, capped, and stored
/// (spec FR-025/026).
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
    let cover_key = match cover_bytes {
        Some(bytes) => {
            if bytes.len() > MAX_COVER_BYTES {
                return Err(ApplicationError::PayloadTooLarge);
            }
            cover_validator
                .sniff(&bytes)
                .await
                .map_err(|_| ApplicationError::UnsupportedMedia)?;
            Some(
                store
                    .put(&bytes)
                    .await
                    .map_err(ApplicationError::Storage)?
                    .storage_key,
            )
        }
        None => None,
    };
    programs
        .insert(
            title.trim().to_owned(),
            description
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_owned),
            cover_key,
            clock.now().await,
        )
        .await
        .map_err(ApplicationError::Repository)
}
