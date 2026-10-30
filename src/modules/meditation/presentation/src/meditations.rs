//! Meditation metadata handlers: daily suggestion, library, detail,
//! description update, and delete.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::commands::delete_meditation::DeleteMeditationCommand;
use meditation_application::commands::rename_meditation::RenameMeditationCommand;
use meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionCommand;
use meditation_application::queries::get_daily_suggestion::GetDailySuggestionQuery;
use meditation_application::queries::get_meditation_detail::GetMeditationDetailQuery;
use meditation_application::queries::list_library::ListLibraryQuery;

use crate::meditation_response::{
    caller_favorite_ids, meditation_response, with_playback, MeditationResponse, ProgramLabel,
    ProgramSummary,
};
use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

/// `GET /meditation/daily-suggestion` — today's suggested meditation, the
/// same pick for everyone all day; `204` when the library is empty.
pub async fn daily_suggestion(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Response, ApiError> {
    let found = deps
        .get_daily_suggestion
        .handle(GetDailySuggestionQuery)
        .await
        .map_err(ApiError::Application)?;
    let Some(found) = found else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };
    let favorite_ids = caller_favorite_ids(&deps, actor.user_id()).await;
    let mut response = with_playback(
        deps.get_playback_state.as_ref(),
        actor.user_id(),
        &favorite_ids,
        &found.meditation,
    )
    .await;
    if let Some(program) = &found.program {
        response.program_id = Some(program.id);
        response.program_has_cover = program.cover_storage_key.is_some();
        response.program = Some(ProgramSummary {
            id: program.id,
            title: program.title.clone(),
            has_cover: response.program_has_cover,
        });
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
    let updated = deps
        .update_meditation_description
        .handle(UpdateMeditationDescriptionCommand {
            meditation_id: id,
            description: request.description,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(meditation_response(&updated)))
}

#[derive(serde::Deserialize)]
pub struct RenameRequest {
    pub name: String,
}

/// `PUT /meditation/meditations/{id}/name` — admin rename.
pub async fn rename_meditation(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<RenameRequest>,
) -> Result<Json<MeditationResponse>, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let renamed = deps
        .rename_meditation
        .handle(RenameMeditationCommand {
            meditation_id: id,
            name: request.name,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(meditation_response(&renamed)))
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
    let listed = deps
        .list_library
        .handle(ListLibraryQuery {
            query: q.map(str::to_owned),
        })
        .await
        .map_err(ApiError::Application)?;
    let favorite_ids = caller_favorite_ids(&deps, actor.user_id()).await;
    let mut responses = Vec::with_capacity(listed.len());
    for entry in &listed {
        let mut response = with_playback(
            deps.get_playback_state.as_ref(),
            actor.user_id(),
            &favorite_ids,
            &entry.meditation,
        )
        .await;
        response.program_id = entry.program_id;
        response.program_has_cover = entry.program_has_cover;
        response.programs = Some(
            entry
                .programs
                .iter()
                .map(|program| ProgramLabel {
                    id: program.id,
                    title: program.title.clone(),
                })
                .collect(),
        );
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
    let found = deps
        .get_meditation_detail
        .handle(GetMeditationDetailQuery { meditation_id: id })
        .await
        .map_err(ApiError::Application)?;
    let favorite_ids = caller_favorite_ids(&deps, actor.user_id()).await;
    let mut response = with_playback(
        deps.get_playback_state.as_ref(),
        actor.user_id(),
        &favorite_ids,
        &found.meditation,
    )
    .await;
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
    deps.delete_meditation
        .handle(DeleteMeditationCommand { meditation_id: id })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[path = "meditations.tests.rs"]
mod tests;
