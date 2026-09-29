use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::{
    body::Body as AxumBody,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
use kernel::{Actor, Role};
use serde::{Deserialize, Serialize};
use soundscape_application::{
    commands,
    ports::{
        media::{AudioValidator, BlobReader, BlobStore, OpusConverter},
        CategoryRepository, Clock, MixRepository, SoundRepository,
    },
    ApplicationError,
};

pub struct SoundscapeDependencies {
    pub categories: Arc<dyn CategoryRepository>,
    pub sounds: Arc<dyn SoundRepository>,
    pub mixes: Arc<dyn MixRepository>,
    pub clock: Arc<dyn Clock>,
    pub validator: Arc<dyn AudioValidator>,
    pub converter: Arc<dyn OpusConverter>,
    pub store: Arc<dyn BlobStore>,
    pub medioss_origin: Arc<String>,
}
pub type SharedDependencies = Arc<SoundscapeDependencies>;

struct CurrentActor(Actor);
impl<S: Send + Sync> FromRequestParts<S> for CurrentActor {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .map(Self)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
struct AdminActor(Actor);
impl<S: Send + Sync> FromRequestParts<S> for AdminActor {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let actor = parts
            .extensions
            .get::<Actor>()
            .cloned()
            .ok_or(StatusCode::UNAUTHORIZED)?;
        if actor.role() == Role::Admin {
            Ok(Self(actor))
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}
fn origin(headers: HeaderMap, expected: &str) -> Result<(), StatusCode> {
    if headers.get("origin").and_then(|value| value.to_str().ok()) == Some(expected) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[derive(Serialize)]
struct CategoryDto {
    id: i64,
    name: String,
    slug: String,
}
impl From<soundscape_domain::SoundCategory> for CategoryDto {
    fn from(value: soundscape_domain::SoundCategory) -> Self {
        Self {
            id: value.id,
            name: value.name,
            slug: value.slug,
        }
    }
}
#[derive(Serialize)]
struct SoundDto {
    id: i64,
    name: String,
    category_id: i64,
    content_type: String,
    size_bytes: i64,
    duration_ms: i64,
    created_at: String,
    builtin: bool,
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
        }
    }
}
#[derive(Deserialize)]
struct Name {
    name: String,
}
#[derive(Deserialize)]
struct UpdateSound {
    name: String,
    category_id: i64,
}
#[derive(Deserialize)]
struct MixInput {
    name: String,
    items: Vec<soundscape_domain::MixItem>,
}
#[derive(Serialize)]
struct MixDto {
    id: i64,
    name: String,
    items: Vec<soundscape_domain::MixItem>,
    created_at: String,
    updated_at: String,
}
impl From<soundscape_domain::Mix> for MixDto {
    fn from(value: soundscape_domain::Mix) -> Self {
        Self {
            id: value.id,
            name: value.name,
            items: value.items,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

pub fn soundscape_router() -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/soundscape/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/soundscape/categories/{id}",
            put(rename_category).delete(delete_category),
        )
        .route(
            "/soundscape/sounds",
            get(list_sounds)
                .post(upload_sound)
                .layer(axum::extract::DefaultBodyLimit::max(
                    commands::sounds::MAX_UPLOAD_BYTES,
                )),
        )
        .route(
            "/soundscape/sounds/{id}",
            put(update_sound).delete(delete_sound),
        )
        .route("/soundscape/sounds/{id}/audio", get(stream_sound))
        .route("/soundscape/mixes", get(list_mixes).post(create_mix))
        .route(
            "/soundscape/mixes/{id}",
            get(get_mix).put(update_mix).delete(delete_mix),
        )
}

async fn list_categories(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<CategoryDto>>, StatusCode> {
    soundscape_application::queries::list_categories(deps.categories.as_ref())
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn create_category(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Json(input): Json<Name>,
) -> Result<(StatusCode, Json<CategoryDto>), StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    commands::create_category(deps.categories.as_ref(), &input.name)
        .await
        .map(|value| (StatusCode::CREATED, Json(value.into())))
        .map_err(map_error)
}
async fn rename_category(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Json(input): Json<Name>,
) -> Result<Json<CategoryDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    commands::rename_category(deps.categories.as_ref(), id, &input.name)
        .await
        .map(|value| Json(value.into()))
        .map_err(map_error)
}
async fn delete_category(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
) -> Result<StatusCode, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    commands::delete_category(deps.categories.as_ref(), id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_error)
}
async fn list_sounds(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<SoundDto>>, StatusCode> {
    soundscape_application::queries::list_sounds(deps.sounds.as_ref())
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn upload_sound(
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
                // Read in bounded chunks instead of field.bytes(): DefaultBodyLimit
                // aborts the stream when the declared limit is hit, which lets us
                // answer 413 (too large) instead of a generic 400 (bad multipart).
                let mut data = Vec::new();
                let mut too_large = false;
                loop {
                    match field.chunk().await {
                        Ok(Some(chunk)) => {
                            data.extend_from_slice(&chunk);
                            if data.len() > commands::sounds::MAX_UPLOAD_BYTES {
                                too_large = true;
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(_) => {
                            // The stream broke at the body limit on an already
                            // large payload means size, not shape; a small
                            // payload failing here really is malformed.
                            return Err(if data.len() >= 2 * 1024 * 1024 {
                                StatusCode::PAYLOAD_TOO_LARGE
                            } else {
                                StatusCode::BAD_REQUEST
                            });
                        }
                    }
                }
                if too_large {
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
                bytes = Some(data);
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
        &name.ok_or(StatusCode::BAD_REQUEST)?,
        category_id.ok_or(StatusCode::BAD_REQUEST)?,
        bytes.ok_or(StatusCode::BAD_REQUEST)?,
    )
    .await
    .map_err(map_error)?;
    Ok((StatusCode::CREATED, Json(sound.into())))
}
async fn update_sound(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Json(input): Json<UpdateSound>,
) -> Result<Json<SoundDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.sounds
        .update(id, &input.name, input.category_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|sound| Json(sound.into()))
        .ok_or(StatusCode::NOT_FOUND)
}
async fn delete_sound(
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
async fn stream_sound(
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
                .into_response())
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
pub(crate) fn parse_range(value: &str, total: u64) -> Option<Result<(u64, u64), ()>> {
    let spec = value.trim().strip_prefix("bytes=")?;
    if spec.contains(',') {
        return None;
    }
    let (start, end) = spec.split_once('-')?;
    match (start.trim(), end.trim()) {
        ("", suffix) => {
            let suffix = suffix.parse::<u64>().ok()?;
            if suffix == 0 || total == 0 {
                Some(Err(()))
            } else {
                Some(Ok((total - suffix.min(total), total - 1)))
            }
        }
        (start, "") => {
            let start = start.parse::<u64>().ok()?;
            if start >= total {
                Some(Err(()))
            } else {
                Some(Ok((start, total - 1)))
            }
        }
        (start, end) => {
            let start = start.parse::<u64>().ok()?;
            let end = end.parse::<u64>().ok()?;
            if total == 0 || start >= total || start > end {
                Some(Err(()))
            } else {
                Some(Ok((start, end.min(total - 1))))
            }
        }
    }
}
async fn list_mixes(
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
) -> Result<Json<Vec<MixDto>>, StatusCode> {
    deps.mixes
        .list_for(actor.user_id())
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn get_mix(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
) -> Result<Json<MixDto>, StatusCode> {
    deps.mixes
        .find_for(id, actor.user_id())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|value| Json(value.into()))
        .ok_or(StatusCode::NOT_FOUND)
}
async fn create_mix(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Json(input): Json<MixInput>,
) -> Result<(StatusCode, Json<MixDto>), StatusCode> {
    origin(headers, &deps.medioss_origin)?;
    commands::mixes::create_mix(
        deps.mixes.as_ref(),
        deps.sounds.as_ref(),
        deps.clock.as_ref(),
        actor.user_id(),
        &input.name,
        input.items,
    )
    .await
    .map(|value| (StatusCode::CREATED, Json(value.into())))
    .map_err(map_error)
}
async fn update_mix(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Json(input): Json<MixInput>,
) -> Result<Json<MixDto>, StatusCode> {
    origin(headers, &deps.medioss_origin)?;
    commands::mixes::update_mix(
        deps.mixes.as_ref(),
        deps.sounds.as_ref(),
        deps.clock.as_ref(),
        id,
        actor.user_id(),
        &input.name,
        input.items,
    )
    .await
    .map(|value| Json(value.into()))
    .map_err(map_error)
}
async fn delete_mix(
    Path(id): Path<i64>,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
) -> Result<StatusCode, StatusCode> {
    origin(headers, &deps.medioss_origin)?;
    commands::mixes::delete_mix(deps.mixes.as_ref(), id, actor.user_id())
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_error)
}
fn admin(actor: &Actor) -> Result<(), StatusCode> {
    if actor.role() == Role::Admin {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
fn map_error(error: ApplicationError) -> StatusCode {
    match error {
        ApplicationError::NotFound => StatusCode::NOT_FOUND,
        ApplicationError::Duplicate | ApplicationError::CategoryInUse => StatusCode::CONFLICT,
        ApplicationError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        ApplicationError::UnsupportedMedia => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApplicationError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
