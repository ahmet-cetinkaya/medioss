use axum::{
    body::Body as AxumBody,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use soundscape_application::{commands, ports::media::BlobReader};

use crate::actors::{admin, AdminActor, CurrentActor};
use crate::dependencies::SharedDependencies;
use crate::error_mapping::status_for;
use crate::multipart_field::read_field;
use crate::origin_guard::origin;
use crate::sounds::SoundDto;

pub(crate) async fn upload_icon(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    mut multipart: Multipart,
) -> Result<Json<SoundDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    let mut icon = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if field.name() == Some("icon") {
            icon = Some(read_field(&mut field, soundscape_domain::MAX_ICON_BYTES).await?);
        }
    }
    commands::sounds::set_sound_icon(
        deps.sounds.as_ref(),
        deps.store.as_ref(),
        deps.image_normalizer.as_ref(),
        id,
        icon.ok_or(StatusCode::BAD_REQUEST)?,
    )
    .await
    .map(|sound| Json(sound.into()))
    .map_err(status_for)
}
pub(crate) async fn clear_icon(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
) -> Result<Json<SoundDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    commands::sounds::clear_sound_icon(deps.sounds.as_ref(), deps.store.as_ref(), id)
        .await
        .map(|sound| Json(sound.into()))
        .map_err(status_for)
}
/// GET /soundscape/sounds/{id}/icon — the uploaded icon image; 404 when the
/// sound has none. Sounds are host content, so any signed-in user may read.
pub(crate) async fn get_icon(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    CurrentActor(_actor): CurrentActor,
) -> Result<Response, StatusCode> {
    let icon = soundscape_application::queries::find_sound_icon(deps.sounds.as_ref(), id)
        .await
        .map_err(status_for)?;
    let (BlobReader(reader), _, total) = deps
        .store
        .open_range(
            &icon.storage_key,
            0,
            soundscape_domain::MAX_ICON_BYTES as u64 + 1,
        )
        .await
        .map_err(|error| {
            if error.starts_with("no blob at ") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    use tokio::io::AsyncReadExt;
    use tokio_util::compat::FuturesAsyncReadCompatExt;
    let mut bytes =
        Vec::with_capacity(total.min(soundscape_domain::MAX_ICON_BYTES as u64) as usize);
    // The reader is bounded locally too: a store that ignores the requested
    // range (or a blob changed out-of-band) cannot force unbounded buffering.
    reader
        .compat()
        .take(soundscape_domain::MAX_ICON_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // A stored icon larger than the upload limit means the media store was
    // changed out-of-band; refuse instead of buffering an arbitrary blob.
    if bytes.len() > soundscape_domain::MAX_ICON_BYTES {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let mut response = Response::new(AxumBody::from(bytes));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&icon.content_type).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    Ok(response)
}
