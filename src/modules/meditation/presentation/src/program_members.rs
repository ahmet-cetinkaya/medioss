//! Program membership handlers (spec FR-027): list, add, reorder, remove.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands::add_member::AddMember;
use meditation_application::commands::remove_member::RemoveMember;
use meditation_application::commands::update_program_order::UpdateProgramOrder;
use meditation_application::queries::get_meditation::GetMeditation;
use meditation_application::queries::list_program_members::ListProgramMembers;
use serde::{Deserialize, Serialize};

use crate::meditation_response::{with_playback, MeditationResponse};
use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

/// `GET /meditation/programs/{id}/members` — the program's meditations, in
/// program order, each with the caller's own progress.
pub async fn program_members(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<MeditationResponse>>, ApiError> {
    deps.programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;
    let members = deps
        .list_program_members
        .handle(ListProgramMembers { program_id: id })
        .await
        .map_err(ApiError::Application)?;
    let mut responses = Vec::with_capacity(members.len());
    for member in &members {
        let meditation = deps
            .get_meditation
            .handle(GetMeditation {
                meditation_id: member.meditation_id,
            })
            .await
            .map_err(ApiError::Application)?;
        responses.push(with_playback(deps.playback.as_ref(), actor.user_id(), &meditation).await);
    }
    Ok(Json(responses))
}

#[derive(Deserialize)]
pub struct MemberRequest {
    pub meditation_id: i64,
}

/// `POST /meditation/programs/{id}/members` — admin add (spec FR-027).
pub async fn add_member(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<MemberRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    deps.add_member
        .handle(AddMember {
            program_id: id,
            meditation_id: request.meditation_id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct MemberOrderRequest {
    pub meditation_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct MemberOrderResponse {
    pub program_id: i64,
    pub meditation_ids: Vec<i64>,
}

/// `PUT /meditation/programs/{id}/meditations/order` — admin reorder.
pub async fn update_order(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<MemberOrderRequest>,
) -> Result<Json<MemberOrderResponse>, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    deps.update_program_order
        .handle(UpdateProgramOrder {
            program_id: id,
            meditation_ids: request.meditation_ids.clone(),
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(MemberOrderResponse {
        program_id: id,
        meditation_ids: request.meditation_ids,
    }))
}

/// `DELETE /meditation/programs/{id}/members/{meditation_id}` — admin
/// remove (spec FR-027).
pub async fn remove_member(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path((id, meditation_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    deps.remove_member
        .handle(RemoveMember {
            program_id: id,
            meditation_id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}
