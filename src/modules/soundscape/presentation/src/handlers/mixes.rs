use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use soundscape_application::commands::create_mix::CreateMix;
use soundscape_application::commands::delete_mix::DeleteMix;
use soundscape_application::commands::update_mix::UpdateMix;

use crate::extractors::CurrentActor;
use crate::router::SharedDependencies;
use crate::support::error_mapping::status_for;
use crate::support::origin_guard::origin;

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
    State(deps): State<SharedDependencies>,
    CurrentActor(actor): CurrentActor,
    Path(id): Path<i64>,
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
    deps.create_mix
        .handle(CreateMix {
            user_id: actor.user_id(),
            name: input.name,
            items: input.items,
        })
        .await
        .map(|value| (StatusCode::CREATED, Json(value.into())))
        .map_err(status_for)
}

async fn update_mix(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Path(id): Path<i64>,
    Json(input): Json<MixInput>,
) -> Result<Json<MixDto>, StatusCode> {
    origin(headers, &deps.medioss_origin)?;
    deps.update_mix
        .handle(UpdateMix {
            id,
            user_id: actor.user_id(),
            name: input.name,
            items: input.items,
        })
        .await
        .map(|value| Json(value.into()))
        .map_err(status_for)
}

async fn delete_mix(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    CurrentActor(actor): CurrentActor,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    origin(headers, &deps.medioss_origin)?;
    deps.delete_mix
        .handle(DeleteMix {
            id,
            user_id: actor.user_id(),
        })
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(status_for)
}

pub(crate) fn register(router: Router<SharedDependencies>) -> Router<SharedDependencies> {
    router
        .route("/soundscape/mixes", get(list_mixes).post(create_mix))
        .route(
            "/soundscape/mixes/{id}",
            get(get_mix).put(update_mix).delete(delete_mix),
        )
}

#[cfg(test)]
#[path = "mixes.tests.rs"]
mod tests;
