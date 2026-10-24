//! Audio streaming and download: Range-aware blob responses (spec FR-009..013, FR-040).
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use meditation_application::queries::get_meditation::GetMeditationQuery;

use crate::{ApiError, CurrentActor, SharedDependencies};
use http_range::{parse_range, Unsatisfiable};

/// `GET /meditation/{id}/audio` — Range-aware streaming.
pub async fn audio(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let meditation = deps
        .get_meditation
        .handle(GetMeditationQuery { meditation_id: id })
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
    let meditation = deps
        .get_meditation
        .handle(GetMeditationQuery { meditation_id: id })
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

#[cfg(test)]
#[path = "audio_stream.tests.rs"]
mod tests;
