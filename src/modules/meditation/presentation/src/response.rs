//! Error mapping and the same-origin guard.

use axum::{
    extract::multipart::Field,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::ports::{StagedMedia, StagingError, UploadStaging};
use meditation_application::ApplicationError;

/// Reads one multipart field's bytes, rejecting it once it exceeds `cap`
/// and surfacing any mid-stream read error as `400` instead of silently
/// treating a truncated body as a complete one.
pub async fn read_field_capped(field: &mut Field<'_>, cap: usize) -> Result<Vec<u8>, ApiError> {
    let mut buffer = Vec::new();
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                buffer.extend_from_slice(&chunk);
                if buffer.len() > cap {
                    return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                }
            }
            Ok(None) => return Ok(buffer),
            Err(_) => return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST)),
        }
    }
}

/// Streams one multipart file field to disk through `staging`, so the upload
/// never sits in memory. Mirrors [`read_field_capped`]: `413` once the field
/// exceeds `cap`, `400` for a mid-stream read error. The staged file is
/// deleted on every error path. `507` answers before any byte is read when the
/// staging volume cannot hold `cap` bytes.
pub async fn stage_field_capped(
    field: &mut Field<'_>,
    staging: &dyn UploadStaging,
    cap: usize,
) -> Result<StagedMedia, ApiError> {
    ensure_free_space(staging, cap).await?;
    let mut sink = staging.open().await.map_err(staging_failure)?;
    let mut received = 0usize;
    loop {
        match field.chunk().await {
            Ok(Some(chunk)) => {
                received += chunk.len();
                if received > cap {
                    return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                }
                sink.append(&chunk).await.map_err(staging_failure)?;
            }
            Ok(None) => return sink.keep().await.map_err(staging_failure),
            Err(_) => return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST)),
        }
    }
}

async fn ensure_free_space(staging: &dyn UploadStaging, cap: usize) -> Result<(), ApiError> {
    match staging.free_bytes().await {
        Ok(free) if free < cap as u64 => Err(ApiError::InsufficientStorage),
        Ok(_) => Ok(()),
        Err(StagingError::Unsupported) => {
            static WARN_ONCE: std::sync::Once = std::sync::Once::new();
            WARN_ONCE.call_once(|| {
                tracing::warn!(
                    "free disk space cannot be measured here; uploads are not preflighted"
                );
            });
            Ok(())
        }
        Err(error) => Err(staging_failure(error)),
    }
}

fn staging_failure(error: StagingError) -> ApiError {
    tracing::error!(%error, "upload staging failed");
    ApiError::StatusCode(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Typed handler error.
pub enum ApiError {
    StatusCode(StatusCode),
    /// The staging volume cannot hold an upload of the configured cap (`507`).
    InsufficientStorage,
    Application(ApplicationError),
}

impl ApiError {
    pub fn unauthorized() -> Self {
        Self::StatusCode(StatusCode::UNAUTHORIZED)
    }

    pub fn forbidden() -> Self {
        Self::StatusCode(StatusCode::FORBIDDEN)
    }
}

/// Maps application errors (including every cover normalizer outcome) onto
/// HTTP statuses.
pub(crate) fn status_for(error: &ApplicationError) -> StatusCode {
    match error {
        ApplicationError::UnsupportedMedia => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApplicationError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        ApplicationError::Busy => StatusCode::SERVICE_UNAVAILABLE,
        ApplicationError::NotFound => StatusCode::NOT_FOUND,
        ApplicationError::DuplicateCategory => StatusCode::CONFLICT,
        ApplicationError::InvalidInput(_)
        | ApplicationError::InvalidDescription
        | ApplicationError::InvalidProgramOrder => StatusCode::BAD_REQUEST,
        ApplicationError::Storage(_) | ApplicationError::Repository(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Maps application errors onto the API contract (404 stays generic, no
/// internal detail leaks).
pub fn application_error_response(error: ApplicationError) -> Response {
    let status = status_for(&error);
    let message = match &error {
        ApplicationError::Repository(_) | ApplicationError::Storage(_) => {
            "internal error".to_owned()
        }
        other => other.to_string(),
    };
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::StatusCode(status) => (status, "").into_response(),
            Self::InsufficientStorage => (
                StatusCode::INSUFFICIENT_STORAGE,
                Json(serde_json::json!({ "error": "insufficient storage for upload" })),
            )
                .into_response(),
            Self::Application(error) => application_error_response(error),
        }
    }
}

/// Strict same-origin guard for state-changing routes, mirroring
/// identity's router: a request is accepted only when its Origin header is
/// present and matches exactly; a missing or mismatched Origin is rejected
/// (fail closed).
fn origin_matches(value: &str, origin: &str) -> bool {
    if value.eq_ignore_ascii_case(origin) {
        return true;
    }
    // Dev-only extra origin (the Astro dev server on :4321 proxying to this
    // API). Fail-closed: unset/empty env keeps single-origin behavior.
    if let Ok(extra) = std::env::var("MEDIOSS_DEV_ORIGIN") {
        let extra = extra.trim();
        if !extra.is_empty() && value.eq_ignore_ascii_case(extra) {
            return true;
        }
    }
    false
}
pub fn ensure_browser_origin(headers: &HeaderMap, expected: &str) -> Result<(), ApiError> {
    let origin = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if origin_matches(origin, expected) {
        return Ok(());
    }
    Err(ApiError::StatusCode(StatusCode::FORBIDDEN))
}

#[cfg(test)]
#[path = "response.tests.rs"]
mod tests;
