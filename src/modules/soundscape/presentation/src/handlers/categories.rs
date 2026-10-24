use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use soundscape_application::commands::create_category::CreateCategory;
use soundscape_application::commands::delete_category::DeleteCategory;
use soundscape_application::commands::rename_category::RenameCategory;
use soundscape_application::queries::list_categories::ListCategories;

use crate::extractors::{AdminActor, CurrentActor};
use crate::router::SharedDependencies;
use crate::support::error_mapping::status_for;
use crate::support::origin_guard::origin;

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

async fn list_categories(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<CategoryDto>>, StatusCode> {
    deps.list_categories
        .handle(ListCategories {})
        .await
        .map(|items| Json(items.into_iter().map(Into::into).collect()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_category(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Json(input): Json<Name>,
) -> Result<(StatusCode, Json<CategoryDto>), StatusCode> {
    crate::extractors::admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.create_category
        .handle(CreateCategory { name: input.name })
        .await
        .map(|value| (StatusCode::CREATED, Json(value.into())))
        .map_err(status_for)
}

async fn rename_category(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Path(id): Path<i64>,
    Json(input): Json<Name>,
) -> Result<Json<CategoryDto>, StatusCode> {
    crate::extractors::admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.rename_category
        .handle(RenameCategory {
            id,
            name: input.name,
        })
        .await
        .map(|value| Json(value.into()))
        .map_err(status_for)
}

async fn delete_category(
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    AdminActor(actor): AdminActor,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    crate::extractors::admin(&actor)?;
    origin(headers, &deps.medioss_origin)?;
    deps.delete_category
        .handle(DeleteCategory { id })
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(status_for)
}

pub(crate) fn register(router: Router<SharedDependencies>) -> Router<SharedDependencies> {
    router
        .route(
            "/soundscape/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/soundscape/categories/{id}",
            put(rename_category).delete(delete_category),
        )
}

#[cfg(test)]
#[path = "categories.tests.rs"]
mod tests;
