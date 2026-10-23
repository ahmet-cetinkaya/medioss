use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, put},
    Json, Router,
};
use serde::Deserialize;
use soundscape_application::commands::delete_sound::DeleteSound;
use soundscape_application::commands::update_sound::UpdateSound;

use crate::extractors::{admin, AdminActor};
use crate::handlers::sounds::SoundDto;
use crate::router::SharedDependencies;
use crate::support::error_mapping::status_for;
use crate::support::origin_guard::origin;

#[derive(Deserialize)]
struct UpdateSoundInput {
    name: String,
    category_id: i64,
    /// Replaces the stored glyph key; absent, null or blank clears it.
    #[serde(default)]
    icon_key: Option<String>,
}

async fn update_sound(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Path(id): Path<i64>,
    Json(input): Json<UpdateSoundInput>,
) -> Result<Json<SoundDto>, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.update_sound
        .handle(UpdateSound {
            id,
            name: input.name,
            category_id: input.category_id,
            icon_key: input.icon_key,
        })
        .await
        .map(|sound| Json(sound.into()))
        .map_err(status_for)
}

async fn delete_sound(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.delete_sound
        .handle(DeleteSound { id })
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(status_for)
}

pub(crate) fn register(router: Router<SharedDependencies>) -> Router<SharedDependencies> {
    router
        .route("/soundscape/sounds/{id}", put(update_sound))
        .route("/soundscape/sounds/{id}", delete(delete_sound))
}
