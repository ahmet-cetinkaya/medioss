use axum::{
    body::Body as AxumBody,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use soundscape_application::{commands, ports::media::BlobReader};

use crate::actors::{admin, AdminActor, CurrentActor};
use crate::byte_range::parse_range;
use crate::dependencies::SharedDependencies;
use crate::error_mapping::map_error;
use crate::multipart_field::read_field;
use crate::origin_guard::origin;

#[derive(Serialize)]
pub(crate) struct SoundDto {
    id: i64,
    name: String,
    category_id: i64,
    content_type: String,
    size_bytes: i64,
    duration_ms: i64,
    created_at: String,
    builtin: bool,
    icon_key: Option<String>,
    has_icon: bool,
}
impl From<soundscape_domain::Sound> for SoundDto {
    fn from(value: soundscape_domain::Sound) -> Self {
        Self {
            id: value.id,
            name: value.name,
            category_id: value.category_id,
            content_type: value.content_type,
            size_bytes: value.size_bytes,
            duration_ms: value.duration_ms,
            created_at: value.created_at.to_rfc3339(),
            builtin: value.builtin,
            icon_key: value.icon_key,
            has_icon: value.icon.is_some(),
        }
    }
}
#[derive(Deserialize)]
pub(crate) struct UpdateSound {
    name: String,
    category_id: i64,
    /// Replaces the stored glyph key; absent, null or blank clears it.
    #[serde(default)]
    icon_key: Option<String>,
}

/// GET /soundscape/sounds/{id}/download — the stored audio as an attachment
/// named after the sound. Extension follows the stored content type.
pub(crate) async fn download_sound(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    CurrentActor(_actor): CurrentActor,
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

pub(crate) async fn list_sounds(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<SoundDto>>, StatusCode> {
    soundscape_application::queries::list_sounds(deps.sounds.as_ref())
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
pub(crate) async fn upload_sound(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<SoundDto>), StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    let mut name = None;
    let mut category_id = None;
    let mut bytes = None;
    let mut icon_key = None;
    let mut icon = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        match field.name() {
            Some("name") => name = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?),
            Some("category_id") => {
                category_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .parse()
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
            }
            Some("file") => {
                let data = read_field(&mut field, commands::sounds::MAX_UPLOAD_BYTES).await?;
                bytes = Some(data);
            }
            Some("icon_key") => {
                icon_key = Some(
                    String::from_utf8(
                        read_field(&mut field, 256)
                            .await
                            .map_err(|_| StatusCode::BAD_REQUEST)?,
                    )
                    .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
            }
            Some("icon") => {
                icon = Some(read_field(&mut field, soundscape_domain::MAX_ICON_BYTES).await?)
            }
            _ => {}
        }
    }
    let sound = commands::sounds::create_sound(
        deps.categories.as_ref(),
        deps.sounds.as_ref(),
        deps.validator.as_ref(),
        deps.converter.as_ref(),
        deps.store.as_ref(),
        deps.image_normalizer.as_ref(),
        &name.ok_or(StatusCode::BAD_REQUEST)?,
        category_id.ok_or(StatusCode::BAD_REQUEST)?,
        bytes.ok_or(StatusCode::BAD_REQUEST)?,
        icon_key.as_deref(),
        icon,
    )
    .await
    .map_err(map_error)?;
    Ok((StatusCode::CREATED, Json(sound.into())))
}
pub(crate) async fn update_sound(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Json(input): Json<UpdateSound>,
) -> Result<Json<SoundDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    commands::sounds::update_sound(
        deps.categories.as_ref(),
        deps.sounds.as_ref(),
        id,
        &input.name,
        input.category_id,
        input.icon_key.as_deref(),
    )
    .await
    .map(|sound| Json(sound.into()))
    .map_err(map_error)
}
pub(crate) async fn delete_sound(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
) -> Result<StatusCode, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    commands::sounds::delete_sound(deps.sounds.as_ref(), deps.store.as_ref(), id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_error)
}
pub(crate) async fn stream_sound(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    CurrentActor(_actor): CurrentActor,
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
