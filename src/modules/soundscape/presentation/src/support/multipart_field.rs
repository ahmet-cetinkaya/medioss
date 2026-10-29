use axum::http::StatusCode;
use soundscape_application::ports::staged_upload::{StagedMedia, StagingError, UploadStaging};

/// Reads one multipart field in bounded chunks instead of field.bytes():
/// DefaultBodyLimit aborts the stream when the declared limit is hit, which
/// lets us answer 413 (too large) instead of a generic 400 (bad multipart).
pub(crate) async fn read_field(
    field: &mut axum::extract::multipart::Field<'_>,
    max: usize,
) -> Result<Vec<u8>, StatusCode> {
    let mut data = Vec::new();
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                data.extend_from_slice(&chunk);
                if data.len() > max {
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
            Ok(None) => return Ok(data),
            Err(_) => {
                // The stream broke at the body limit on an already large
                // payload means size, not shape; a small payload failing
                // here really is malformed.
                return Err(if data.len() >= 2 * 1024 * 1024 {
                    StatusCode::PAYLOAD_TOO_LARGE
                } else {
                    StatusCode::BAD_REQUEST
                });
            }
        }
    }
}

/// Streams one multipart file field to disk through `staging`, so the upload
/// never sits in memory. `413` once the field exceeds `cap`, `400` for a
/// mid-stream read error, `507` before any byte is read when the staging
/// volume cannot hold `cap` bytes. The staged file is deleted on every error
/// path. Mirrors meditation-presentation's `stage_field_capped`; the boundary
/// rule forbids sharing it across modules.
pub(crate) async fn stage_field(
    field: &mut axum::extract::multipart::Field<'_>,
    staging: &dyn UploadStaging,
    cap: usize,
) -> Result<StagedMedia, StatusCode> {
    ensure_free_space(staging, cap).await?;
    let mut sink = staging.open().await.map_err(staging_failure)?;
    let mut received = 0usize;
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                received += chunk.len();
                if received > cap {
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
                sink.append(&chunk).await.map_err(staging_failure)?;
            }
            Ok(None) => return sink.keep().await.map_err(staging_failure),
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    }
}

async fn ensure_free_space(staging: &dyn UploadStaging, cap: usize) -> Result<(), StatusCode> {
    match staging.free_bytes().await {
        Ok(free) if free < cap as u64 => Err(StatusCode::INSUFFICIENT_STORAGE),
        // Platforms that cannot measure free space skip the preflight.
        Ok(_) | Err(StagingError::Unsupported) => Ok(()),
        Err(StagingError::Io(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn staging_failure(error: StagingError) -> StatusCode {
    match error {
        StagingError::Io(_) | StagingError::Unsupported => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
#[path = "multipart_field.tests.rs"]
mod tests;
