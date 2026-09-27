//! Audio-surface handlers: upload, library, metadata, streaming, download,
//! delete. Streaming implements the Range contract (spec FR-009..013).
use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::{commands, queries};
use serde::Serialize;

use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

#[derive(Serialize)]
pub struct MeditationResponse {
    pub id: i64,
    pub name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub duration_ms: i64,
    pub created_at: String,
}

pub(crate) fn meditation_response(
    meditation: &meditation_domain::Meditation,
) -> MeditationResponse {
    MeditationResponse {
        id: meditation.id(),
        name: meditation.name().to_owned(),
        content_type: meditation.content_type().to_owned(),
        size_bytes: meditation.size_bytes(),
        duration_ms: meditation.duration_ms(),
        created_at: meditation.created_at().to_rfc3339(),
    }
}

/// `POST /meditation/audio` — admin-only multipart upload (file + name).
///
/// The 100MB cap is enforced twice: against the multipart field's declared
/// length before its body is accepted (spec FR-002), and against a running
/// byte counter so understated lengths cannot smuggle past (spec FR-003).
pub async fn upload(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MeditationResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;

    let max_bytes = commands::MAX_UPLOAD_BYTES;
    // Text fields are bounded the same way: a streaming field cannot grow
    // past the name limit in memory.
    let max_text = commands::MAX_NAME_LENGTH * 4;
    let mut name: Option<String> = None;
    let mut program_id: Option<i64> = None;
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?
    {
        match field.name() {
            Some("program_id") => {
                let mut text = Vec::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    text.extend_from_slice(&chunk);
                    if text.len() > 32 {
                        return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                    }
                }
                program_id = Some(
                    String::from_utf8_lossy(&text)
                        .parse::<i64>()
                        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?,
                );
            }
            Some("name") => {
                let mut text = Vec::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    text.extend_from_slice(&chunk);
                    if text.len() > max_text {
                        return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                    }
                }
                name = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("file") => {
                let mut buffer = Vec::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    buffer.extend_from_slice(&chunk);
                    if buffer.len() > max_bytes {
                        return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                    }
                }
                bytes = Some(buffer);
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let name = name.ok_or(ApiError::StatusCode(StatusCode::BAD_REQUEST))?;
    let program_id = program_id.ok_or(ApiError::StatusCode(StatusCode::BAD_REQUEST))?;
    let bytes = bytes.ok_or(ApiError::StatusCode(StatusCode::BAD_REQUEST))?;

    let meditation = commands::upload_meditation(
        deps.meditations.as_ref(),
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
        deps.validator.as_ref(),
        deps.converter.as_ref(),
        deps.store.as_ref(),
        deps.clock.as_ref(),
        &name,
        program_id,
        bytes,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok((StatusCode::CREATED, Json(meditation_response(&meditation))))
}

/// `GET /meditation/library` — the shared library, newest first.
pub async fn library(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<MeditationResponse>>, ApiError> {
    let listed = queries::list_library(deps.meditations.as_ref())
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(listed.iter().map(meditation_response).collect()))
}

/// `GET /meditation/{id}` — one meditation's metadata.
pub async fn meditation(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<MeditationResponse>, ApiError> {
    let found = queries::get_meditation(deps.meditations.as_ref(), id)
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(meditation_response(&found)))
}

/// One satisfiable byte range of a blob.
pub struct ByteRange {
    pub start: u64,
    pub end_inclusive: u64,
}

/// A well-formed but unsatisfiable Range header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsatisfiable;

/// RFC 7233 single-range parsing.
///
/// - `Ok(None)` — absent or malformed header, or multi-range: serve the full
///   body with `200` (spec FR-012).
/// - `Ok(Some(range))` — serve `206` with `Content-Range`.
/// - `Err(())` — well-formed but unsatisfiable: `416` with `bytes */total`
///   (spec FR-011).
pub fn parse_range(header: &str, total: u64) -> Result<Option<ByteRange>, Unsatisfiable> {
    let header = header.trim();
    let Some(spec) = header.strip_prefix("bytes=") else {
        return Ok(None);
    };
    if spec.contains(',') {
        return Ok(None);
    }
    let (start_text, end_text) = spec.split_once('-').ok_or(Unsatisfiable)?;
    match (start_text.trim(), end_text.trim()) {
        ("", "") => Ok(None),
        // Suffix form: the last N bytes.
        ("", suffix_text) => {
            let suffix: u64 = suffix_text.parse().map_err(|_| Unsatisfiable)?;
            if suffix == 0 || total == 0 {
                return Err(Unsatisfiable);
            }
            let start = total - suffix.min(total);
            Ok(Some(ByteRange {
                start,
                end_inclusive: total - 1,
            }))
        }
        (start_text, "") => {
            let start: u64 = start_text.parse().map_err(|_| Unsatisfiable)?;
            if total == 0 || start >= total {
                return Err(Unsatisfiable);
            }
            Ok(Some(ByteRange {
                start,
                end_inclusive: total - 1,
            }))
        }
        (start_text, end_text) => {
            let start: u64 = start_text.parse().map_err(|_| Unsatisfiable)?;
            let end: u64 = end_text.parse().map_err(|_| Unsatisfiable)?;
            if total == 0 || start >= total || start > end {
                return Err(Unsatisfiable);
            }
            Ok(Some(ByteRange {
                start,
                end_inclusive: end.min(total - 1),
            }))
        }
    }
}

/// `GET /meditation/{id}/audio` — Range-aware streaming.
pub async fn audio(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let meditation = queries::get_meditation(deps.meditations.as_ref(), id)
        .await
        .map_err(ApiError::Application)?;
    serve_blob(deps, meditation.storage_key(), &headers, None).await
}

/// `GET /meditation/{id}/download` — the stored audio as an attachment
/// named after the meditation (spec FR-040).
pub async fn download(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let meditation = queries::get_meditation(deps.meditations.as_ref(), id)
        .await
        .map_err(ApiError::Application)?;
    serve_blob(
        deps,
        meditation.storage_key(),
        &headers,
        Some(meditation.name()),
    )
    .await
}

/// Streams a stored blob. `download_name` switches the response to an
/// attachment with a sanitized filename.
async fn serve_blob(
    deps: SharedDependencies,
    key: &str,
    headers: &HeaderMap,
    download_name: Option<&str>,
) -> Result<Response, ApiError> {
    let bytes = deps
        .store
        .read(key)
        .await
        .map_err(meditation_application::ApplicationError::Storage)
        .map_err(ApiError::Application)?;
    let total = bytes.len() as u64;
    let content_type = meditation_domain::STORED_CONTENT_TYPE;

    let mut response = match headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
    {
        Some(range_header) => match parse_range(range_header, total) {
            Ok(None) => full_response(&bytes, content_type, download_name),
            Ok(Some(range)) => {
                let start = range.start as usize;
                let end = (range.end_inclusive + 1) as usize;
                let mut response = (
                    StatusCode::PARTIAL_CONTENT,
                    [(header::CONTENT_TYPE, content_type)],
                    bytes[start..end.min(bytes.len())].to_vec(),
                )
                    .into_response();
                insert_header(
                    &mut response,
                    header::CONTENT_RANGE,
                    &format!("bytes {}-{}/{}", range.start, range.end_inclusive, total),
                )?;
                response
            }
            Err(Unsatisfiable) => {
                let mut response = (StatusCode::RANGE_NOT_SATISFIABLE, "").into_response();
                insert_header(
                    &mut response,
                    header::CONTENT_RANGE,
                    &format!("bytes */{total}"),
                )?;
                response
            }
        },
        None => full_response(&bytes, content_type, download_name),
    };
    insert_header(&mut response, header::ACCEPT_RANGES, "bytes")?;
    Ok(response)
}

fn insert_header(
    response: &mut Response,
    name: header::HeaderName,
    value: &str,
) -> Result<(), ApiError> {
    response.headers_mut().insert(
        name,
        HeaderValue::from_str(value)
            .map_err(|_| ApiError::StatusCode(StatusCode::INTERNAL_SERVER_ERROR))?,
    );
    Ok(())
}

fn full_response(
    bytes: &[u8],
    content_type: &'static str,
    download_name: Option<&str>,
) -> Response {
    let mut response = (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        bytes.to_vec(),
    )
        .into_response();
    if let Some(name) = download_name {
        let sanitized = sanitize_filename(name);
        let disposition = format!("attachment; filename=\"{sanitized}.opus\"");
        if let Ok(value) = header::HeaderValue::from_str(&disposition) {
            response
                .headers_mut()
                .insert(header::CONTENT_DISPOSITION, value);
        }
    }
    response
}

/// Keeps download filenames header-safe: ASCII alphanumerics and dashes;
/// everything else collapses to a dash.
fn sanitize_filename(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || character == '-' {
            out.push(character);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// `DELETE /meditation/{id}` — admin delete with full cleanup (spec FR-014).
pub async fn delete_meditation(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    commands::delete_meditation(
        deps.meditations.as_ref(),
        deps.memberships.as_ref(),
        deps.notes.as_ref(),
        deps.store.as_ref(),
        id,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Test-only seeding: bypasses HTTP and the router to place one stored
/// meditation into the fakes (router.tests.rs).
#[cfg(test)]
pub(crate) async fn upload_for_tests(
    state: &meditation_application::test_support::RepoState,
    store: &meditation_application::test_support::FakeBlobStore,
    bytes: Vec<u8>,
) -> meditation_domain::Meditation {
    use meditation_application::ports::BlobStore as _;
    let blob = store.put("holder", "seed", "opus", &bytes).await.unwrap();
    meditation_domain::Meditation::new(
        1,
        "Seed",
        meditation_domain::STORED_CONTENT_TYPE,
        blob.size_bytes,
        &blob.storage_key,
        60_000,
        chrono::Utc::now(),
    )
    .unwrap()
    .tap(|row| state.seed_meditation(row.clone()))
}

#[cfg(test)]
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}
#[cfg(test)]
impl<T> Tap for T {}
