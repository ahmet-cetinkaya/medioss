//! Playback-state handlers: position, playback-state, completion, toggle
//! (spec FR-015..022). All per-user state is scoped to the acting user.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use meditation_application::commands::record_completion::{
    RecordCompletion, RecordCompletionHandler, RecordCompletionService,
};
use meditation_application::commands::record_position::{
    RecordPosition, RecordPositionHandler, RecordPositionService,
};
use meditation_application::commands::set_completed::{
    SetCompleted, SetCompletedHandler, SetCompletedService,
};
use meditation_application::queries::get_playback_state::{
    GetPlaybackState, GetPlaybackStateHandler, GetPlaybackStateService,
};
use serde::{Deserialize, Serialize};

use crate::{ensure_browser_origin, ApiError, CurrentActor, SharedDependencies};

#[derive(Deserialize)]
pub struct PositionRequest {
    pub position_ms: i64,
}

#[derive(Serialize)]
pub struct PlaybackStateResponse {
    pub position_ms: i64,
    pub completed: bool,
}

/// `GET /meditation/{id}/playback-state` — the caller's own state
/// (spec FR-016).
pub async fn playback_state(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<PlaybackStateResponse>, ApiError> {
    let state = GetPlaybackStateService::new(deps.meditations.clone(), deps.playback.clone())
        .handle(GetPlaybackState {
            user_id: actor.user_id(),
            meditation_id: id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(PlaybackStateResponse {
        position_ms: state.position_ms,
        completed: state.completed,
    }))
}

/// `POST /meditation/{id}/playback-state` — record the caller's position
/// (spec FR-015).
pub async fn position(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<PositionRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    RecordPositionService::new(deps.meditations.clone(), deps.playback.clone())
        .handle(RecordPosition {
            user_id: actor.user_id(),
            meditation_id: id,
            position_ms: request.position_ms,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CompletionRequest {
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct CompletionResponse {
    pub session_id: i64,
    pub started_at: String,
    pub finished_at: String,
}

/// `POST /meditation/{id}/completion` — idempotent completion report
/// (spec FR-020/021).
pub async fn completion(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<CompletionRequest>,
) -> Result<(StatusCode, Json<CompletionResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let session = RecordCompletionService::new(
        deps.meditations.clone(),
        deps.playback.clone(),
        deps.sessions.clone(),
    )
    .handle(RecordCompletion {
        user_id: actor.user_id(),
        meditation_id: id,
        started_at: request.started_at,
        finished_at: request.finished_at,
    })
    .await
    .map_err(ApiError::Application)?;
    Ok((
        StatusCode::CREATED,
        Json(CompletionResponse {
            session_id: session.id,
            started_at: session.started_at.to_rfc3339(),
            finished_at: session.finished_at.to_rfc3339(),
        }),
    ))
}

#[derive(Deserialize)]
pub struct CompletedRequest {
    pub completed: bool,
}

/// `PUT /meditation/{id}/completed` — toggle the caller's completed state;
/// sessions and position stay untouched (spec FR-022).
pub async fn set_completed(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<CompletedRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    SetCompletedService::new(deps.meditations.clone(), deps.playback.clone())
        .handle(SetCompleted {
            user_id: actor.user_id(),
            meditation_id: id,
            completed: request.completed,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}
