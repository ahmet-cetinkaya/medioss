use axum::{
    body::Body as AxumBody,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Response,
    routing::{delete, get, put},
    Json, Router,
};
use soundscape_application::commands::clear_sound_icon::ClearSoundIcon;
use soundscape_application::commands::set_sound_icon::SetSoundIcon;
use soundscape_application::ports::media::BlobReader;
use soundscape_application::queries::find_sound_icon::FindSoundIcon;

use crate::extractors::{admin, AdminActor, CurrentActor};
use crate::handlers::sounds::SoundDto;
use crate::router::SharedDependencies;
use crate::support::error_mapping::status_for;
use crate::support::multipart_field::read_field;
use crate::support::origin_guard::origin;

async fn upload_icon(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Path(id): Path<i64>,
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
    deps.set_sound_icon
        .handle(SetSoundIcon {
            id,
            bytes: icon.ok_or(StatusCode::BAD_REQUEST)?,
        })
        .await
        .map(|sound| Json(sound.into()))
        .map_err(status_for)
}

async fn clear_icon(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Path(id): Path<i64>,
) -> Result<Json<SoundDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.clear_sound_icon
        .handle(ClearSoundIcon { id })
        .await
        .map(|sound| Json(sound.into()))
        .map_err(status_for)
}

/// GET /soundscape/sounds/{id}/icon — the uploaded icon image; 404 when the
/// sound has none. Sounds are host content, so any signed-in user may read.
/// A pure byte read: no use case wraps it, the store port is used directly.
async fn get_icon(
    State(deps): State<SharedDependencies>,
    CurrentActor(_actor): CurrentActor,
    Path(id): Path<i64>,
) -> Result<Response, StatusCode> {
    let icon = deps
        .find_sound_icon
        .handle(FindSoundIcon { id })
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

pub(crate) fn register(router: Router<SharedDependencies>) -> Router<SharedDependencies> {
    router
        .route("/soundscape/sounds/{id}/icon", get(get_icon))
        .route(
            "/soundscape/sounds/{id}/icon",
            put(upload_icon).layer(axum::extract::DefaultBodyLimit::max(
                2 * soundscape_domain::MAX_ICON_BYTES,
            )),
        )
        .route("/soundscape/sounds/{id}/icon", delete(clear_icon))
}

#[cfg(test)]
#[path = "sounds_icon.tests.rs"]
mod tests;
