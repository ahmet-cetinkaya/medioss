use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use soundscape_application::commands;

use crate::actors::CurrentActor;
use crate::dependencies::SharedDependencies;
use crate::error_mapping::map_error;
use crate::origin_guard::origin;

#[derive(Deserialize)]
pub(crate) struct MixInput {
    name: String,
    items: Vec<soundscape_domain::MixItem>,
}
#[derive(Serialize)]
pub(crate) struct MixDto {
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

pub(crate) async fn list_mixes(
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
) -> Result<Json<Vec<MixDto>>, StatusCode> {
    deps.mixes
        .list_for(actor.user_id())
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
pub(crate) async fn get_mix(
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
pub(crate) async fn create_mix(
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
pub(crate) async fn update_mix(
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
pub(crate) async fn delete_mix(
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
