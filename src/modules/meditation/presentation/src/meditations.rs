//! Meditation metadata handlers: daily suggestion, library, detail,
//! description update, and delete.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::{commands, queries};

use crate::meditation_response::{
    meditation_response, with_playback, MeditationResponse, ProgramSummary,
};
use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

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
