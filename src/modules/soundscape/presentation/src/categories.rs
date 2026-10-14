use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use soundscape_application::commands;

use crate::actors::{admin, AdminActor, CurrentActor};
use crate::dependencies::SharedDependencies;
use crate::error_mapping::map_error;
use crate::origin_guard::origin;

#[derive(Serialize)]
pub(crate) struct CategoryDto {
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
#[derive(Deserialize)]
pub(crate) struct Name {
    name: String,
}

pub(crate) async fn list_categories(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<CategoryDto>>, StatusCode> {
    soundscape_application::queries::list_categories(deps.categories.as_ref())
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
pub(crate) async fn create_category(
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
pub(crate) async fn rename_category(
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
pub(crate) async fn delete_category(
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
