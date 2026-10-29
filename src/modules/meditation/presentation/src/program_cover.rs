//! Program cover handlers (spec FR-026): authenticated read and replacement.
use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use meditation_application::commands::remove_program_cover::RemoveProgramCoverCommand;
use meditation_application::commands::replace_program_cover::ReplaceProgramCoverCommand;
use meditation_application::queries::get_program_cover::{GetProgramCoverQuery, ProgramCover};

use crate::response::read_field_capped;
use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

/// `GET /meditation/programs/{id}/cover` — the cover image, authenticated
/// (spec FR-026).
pub async fn cover(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let ProgramCover {
        bytes,
        content_type,
    } = deps
        .get_program_cover
        .handle(GetProgramCoverQuery {
            requester_id: actor.user_id(),
            program_id: id,
        })
        .await
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;
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

    let max_cover = deps.max_cover_bytes;
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
    let Some((bytes, _ext)) = cover else {
        return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST));
    };

    deps.replace_program_cover
        .handle(ReplaceProgramCoverCommand {
            requester_id: _actor.user_id(),
            program_id: id,
            bytes,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /meditation/programs/{id}/cover` — removes the cover image.
/// Idempotent: a program without a cover answers 204 as well.
pub async fn remove_cover(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    deps.remove_program_cover
        .handle(RemoveProgramCoverCommand { program_id: id })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "program_cover.tests.rs"]
mod tests;
