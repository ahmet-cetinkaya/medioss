use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use soundscape_application::ports::media::BlobReader;

use crate::extractors::CurrentActor;
use crate::router::SharedDependencies;
use crate::support::byte_range::parse_range;

async fn stream_sound(
    State(deps): State<SharedDependencies>,
    CurrentActor(_actor): CurrentActor,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let sound = deps
        .sounds
        .find(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let (_, _, total) = deps
        .store
        .open_range(&sound.storage_key, 0, 0)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| parse_range(value, total));
    let (status, start, length, content_range) = match range {
        Some(Ok((start, end))) => (
            StatusCode::PARTIAL_CONTENT,
            start,
            end - start + 1,
            Some(format!("bytes {start}-{end}/{total}")),
        ),
        Some(Err(())) => {
            return Ok((
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{total}"))],
            )
                .into_response());
        }
        None => (StatusCode::OK, 0, total, None),
    };
    let (BlobReader(reader), content_type, _) = deps
        .store
        .open_range(&sound.storage_key, start, length)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    use tokio_util::compat::FuturesAsyncReadCompatExt;
    let mut response = Response::new(AxumBody::from_stream(tokio_util::io::ReaderStream::new(
        reader.compat(),
    )));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    response
        .headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&length.to_string())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    if let Some(value) = content_range {
        response.headers_mut().insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        );
    }
    Ok(response)
}

/// GET /soundscape/sounds/{id}/download — the stored audio as an attachment
/// named after the sound. A pure byte read; no use case wraps it.
async fn download_sound(
    State(deps): State<SharedDependencies>,
    CurrentActor(_actor): CurrentActor,
    Path(id): Path<i64>,
) -> Result<Response, StatusCode> {
    let sound = deps
        .sounds
        .find(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let (BlobReader(reader), _, total) = deps
        .store
        .open_range(&sound.storage_key, 0, u64::MAX)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    use tokio::io::AsyncReadExt;
    use tokio_util::compat::FuturesAsyncReadCompatExt;
    let mut bytes = Vec::with_capacity(total.min(64 * 1024 * 1024) as usize);
    reader
        .compat()
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let name: String = sound
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let name = name.trim_matches('-');
    let extension = match sound.content_type.as_str() {
        "audio/mpeg" => "mp3",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/ogg" => "ogg",
        _ => "bin",
    };
    let disposition = format!("attachment; filename=\"{name}.{extension}\"");
    let mut response = Response::new(AxumBody::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&sound.content_type)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

pub(crate) fn register(router: Router<SharedDependencies>) -> Router<SharedDependencies> {
    router
        .route("/soundscape/sounds/{id}/audio", get(stream_sound))
        .route("/soundscape/sounds/{id}/download", get(download_sound))
}
