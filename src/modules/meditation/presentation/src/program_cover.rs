//! Program cover handlers (spec FR-026): authenticated read and replacement.
use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use meditation_application::commands;

use crate::response::read_field_capped;
use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

/// `GET /meditation/programs/{id}/cover` — the cover image, authenticated
/// (spec FR-026).
pub async fn cover(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let program = deps
        .programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;
    let key = program.cover_storage_key.ok_or(ApiError::Application(
        meditation_application::ApplicationError::NotFound,
    ))?;
    let bytes = deps
        .store
        .read(&key)
        .await
        .map_err(meditation_application::ApplicationError::Storage)
        .map_err(ApiError::Application)?;
    let content_type = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else {
        "image/webp"
    };
    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(content_type),
        )],
        bytes,
    )
        .into_response())
}

/// `PUT /meditation/programs/{id}/cover` (spec FR-026 follow-up: covers are
/// replaceable, not create-only). Removes the previous cover blob, if any.
pub async fn replace_cover(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;

    let max_cover = commands::MAX_COVER_BYTES;
    let mut cover: Option<(Vec<u8>, &'static str)> = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?
    {
        if field.name() == Some("cover") {
            let buffer = read_field_capped(&mut field, max_cover).await?;
            let normalized = deps
                .cover_validator
                .normalize(buffer)
                .await
                .map_err(meditation_application::ApplicationError::from)
                .map_err(ApiError::Application)?;
            cover = Some((normalized, "webp"));
        } else {
            let _ = field.bytes().await;
        }
    }
    let Some((bytes, ext)) = cover else {
        return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST));
    };

    let program = deps
        .programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;

    let blob = deps
        .store
        .put(&program.folder, "cover", ext, &bytes)
        .await
        .map_err(meditation_application::ApplicationError::Storage)
        .map_err(ApiError::Application)?;
    deps.programs
        .set_cover_key(id, &blob.storage_key)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?;
    if let Some(old_key) = program.cover_storage_key {
        if old_key != blob.storage_key {
            let _ = deps.store.delete(&old_key).await;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
