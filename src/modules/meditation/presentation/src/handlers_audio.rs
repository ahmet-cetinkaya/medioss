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
    pub description: Option<String>,
    /// Every meditation surface must show progress: `0` and `false` when the
    /// caller never touched this meditation.
    pub completed: bool,
    pub progress_percent: i64,
    /// The meditation's home program: set on the daily-suggestion and library
    /// responses (cover borrowing, \"no program\" detection); absent elsewhere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_id: Option<i64>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub program_has_cover: bool,
    /// The home program as the Player needs it (cover borrowing, link label);
    /// set on the detail response, absent elsewhere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<ProgramSummary>,
}

/// The part of a program the Player screen renders: its identity, the title
/// for the program link's label, and whether a cover can be borrowed.
#[derive(Serialize)]
pub struct ProgramSummary {
    pub id: i64,
    pub title: String,
    pub has_cover: bool,
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
        description: meditation.description().map(str::to_owned),
        completed: false,
        progress_percent: 0,
        program_id: None,
        program_has_cover: false,
        program: None,
    }
}

/// `GET /meditation/daily-suggestion` — today's suggested meditation, the
/// same pick for everyone all day; `204` when the library is empty.
pub async fn daily_suggestion(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Response, ApiError> {
    let found = queries::get_daily_suggestion(
        deps.meditations.as_ref(),
        deps.clock.as_ref(),
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
    )
    .await
    .map_err(ApiError::Application)?;
    let Some(found) = found else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };
    let mut response =
        with_playback(deps.playback.as_ref(), actor.user_id(), &found.meditation).await;
    if let Some(program) = &found.program {
        response.program_id = Some(program.id);
        response.program_has_cover = program.cover_storage_key.is_some();
    }
    Ok(Json(response).into_response())
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
    let mut description: Option<String> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?
    {
        match field.name() {
            Some("program_id") => {
                let text = crate::response::read_field_capped(&mut field, 32).await?;
                program_id = Some(
                    String::from_utf8_lossy(&text)
                        .parse::<i64>()
                        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?,
                );
            }
            Some("name") => {
                let text = crate::response::read_field_capped(&mut field, max_text).await?;
                name = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("description") => {
                // Cap above the 2000-char limit (4 bytes/char) so an
                // over-long value reaches the command and gets the 400.
                let text = crate::response::read_field_capped(
                    &mut field,
                    meditation_domain::MAX_DESCRIPTION_LENGTH * 4 + 4,
                )
                .await?;
                description = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("file") => {
                bytes = Some(crate::response::read_field_capped(&mut field, max_bytes).await?);
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
        description.as_deref(),
    )
    .await
    .map_err(ApiError::Application)?;
    Ok((StatusCode::CREATED, Json(meditation_response(&meditation))))
}

#[derive(serde::Deserialize)]
pub struct UpdateDescriptionRequest {
    pub description: Option<String>,
}

/// `PUT /meditation/meditations/{id}` — admin set/clear of the description.
pub async fn update_description(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<UpdateDescriptionRequest>,
) -> Result<Json<MeditationResponse>, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let updated = commands::update_meditation_description(
        deps.meditations.as_ref(),
        id,
        request.description.as_deref(),
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(Json(meditation_response(&updated)))
}

/// `GET /meditation/library` — the shared library, newest first.
#[derive(serde::Deserialize)]
pub struct LibraryQuery {
    pub q: Option<String>,
}

const MAX_QUERY_CHARS: usize = 200;

pub async fn library(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    axum::extract::Query(query): axum::extract::Query<LibraryQuery>,
) -> Result<Json<Vec<MeditationResponse>>, ApiError> {
    let q = query.q.as_deref();
    if q.is_some_and(|q| q.chars().count() > MAX_QUERY_CHARS) {
        return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST));
    }
    let listed = queries::list_library(
        deps.meditations.as_ref(),
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
        deps.categories.as_ref(),
        q,
    )
    .await
    .map_err(ApiError::Application)?;
    let mut responses = Vec::with_capacity(listed.len());
    let homes = deps.memberships.home_programs().await.map_err(|e| {
        ApiError::Application(meditation_application::ApplicationError::Repository(e))
    })?;
    for meditation in &listed {
        let mut response = with_playback(deps.playback.as_ref(), actor.user_id(), meditation).await;
        if let Some((program_id, has_cover)) = homes.get(&meditation.id()) {
            response.program_id = Some(*program_id);
            response.program_has_cover = *has_cover;
        }
        responses.push(response);
    }
    Ok(Json(responses))
}

/// `GET /meditation/{id}` — one meditation's metadata, with its home program
/// as a summary the Player can render without further requests.
pub async fn meditation(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<MeditationResponse>, ApiError> {
    let found = queries::get_meditation_detail(
        deps.meditations.as_ref(),
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
        id,
    )
    .await
    .map_err(ApiError::Application)?;
    let mut response =
        with_playback(deps.playback.as_ref(), actor.user_id(), &found.meditation).await;
    if let Some(program) = found.program {
        let has_cover = program.cover_storage_key.is_some();
        response.program_id = Some(program.id);
        response.program_has_cover = has_cover;
        response.program = Some(ProgramSummary {
            id: program.id,
            title: program.title,
            has_cover,
        });
    }
    Ok(Json(response))
}

/// Fills a meditation response's `completed`/`progress_percent` from the
/// caller's own playback state; every meditation surface must show progress,
/// so this is not optional decoration.
pub(crate) async fn with_playback(
    playback: &dyn meditation_application::ports::PlaybackStateRepository,
    user_id: kernel::UserId,
    meditation: &meditation_domain::Meditation,
) -> MeditationResponse {
    let mut response = meditation_response(meditation);
    if let Ok(Some(state)) = playback.get(user_id, meditation.id()).await {
        response.completed = state.completed;
        response.progress_percent = if state.completed {
            100
        } else if meditation.duration_ms() > 0 {
            (((state.position_ms as i128) * 100) / meditation.duration_ms() as i128).clamp(0, 100)
                as i64
        } else {
            0
        };
    }
    response
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
    // A malformed range spec (no `-`, or a non-numeric bound) is not a
    // well-formed-but-unsatisfiable range per FR-012: ignore it and serve
    // the full body, matching absent/multi-range handling above.
    let Some((start_text, end_text)) = spec.split_once('-') else {
        return Ok(None);
    };
    match (start_text.trim(), end_text.trim()) {
        ("", "") => Ok(None),
        // Suffix form: the last N bytes.
        ("", suffix_text) => {
            let Ok(suffix) = suffix_text.parse::<u64>() else {
                return Ok(None);
            };
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
            let Ok(start) = start_text.parse::<u64>() else {
                return Ok(None);
            };
            if total == 0 || start >= total {
                return Err(Unsatisfiable);
            }
            Ok(Some(ByteRange {
                start,
                end_inclusive: total - 1,
            }))
        }
        (start_text, end_text) => {
            let (Ok(start), Ok(end)) = (start_text.parse::<u64>(), end_text.parse::<u64>()) else {
                return Ok(None);
            };
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
    serve_blob(
        deps,
        meditation.storage_key(),
        meditation.content_type(),
        &headers,
        None,
    )
    .await
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
        meditation.content_type(),
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
    content_type: &str,
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

fn full_response(bytes: &[u8], content_type: &str, download_name: Option<&str>) -> Response {
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
