//! Category handlers (spec FR-030..032): list, create, rename, delete, and
//! assigning a program's category. Management routes are admin-gated.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands::create_category::{
    CreateCategory, CreateCategoryHandler, CreateCategoryService,
};
use meditation_application::commands::delete_category::{
    DeleteCategory, DeleteCategoryHandler, DeleteCategoryService,
};
use meditation_application::commands::rename_category::{
    RenameCategory, RenameCategoryHandler, RenameCategoryService,
};
use meditation_application::commands::set_program_category::{
    SetProgramCategory, SetProgramCategoryHandler, SetProgramCategoryService,
};
use meditation_application::queries::list_categories::{
    ListCategories, ListCategoriesHandler, ListCategoriesService,
};
use serde::{Deserialize, Serialize};

use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RenameCategoryRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
}

/// `GET /meditation/categories` — every category.
pub async fn categories(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let listed = ListCategoriesService::new(deps.categories.clone())
        .handle(ListCategories)
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(
        listed
            .into_iter()
            .map(|category| CategoryResponse {
                id: category.id,
                name: category.name,
            })
            .collect(),
    ))
}

/// `POST /meditation/categories` — admin create (spec FR-030).
pub async fn create_category(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    Json(request): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let category = CreateCategoryService::new(deps.categories.clone(), deps.clock.clone())
        .handle(CreateCategory {
            name: request.name.clone(),
        })
        .await
        .map_err(ApiError::Application)?;
    Ok((
        StatusCode::CREATED,
        Json(CategoryResponse {
            id: category.id,
            name: category.name,
        }),
    ))
}

/// `PUT /meditation/categories/{id}` — admin rename; the name stays unique
/// instance-wide.
pub async fn rename_category(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<RenameCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    RenameCategoryService::new(deps.categories.clone(), deps.clock.clone())
        .handle(RenameCategory {
            category_id: id,
            name: request.name.clone(),
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /meditation/categories/{id}` — admin delete; programs become
/// uncategorized (spec FR-032).
pub async fn delete_category(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    DeleteCategoryService::new(deps.categories.clone(), deps.programs.clone())
        .handle(DeleteCategory { category_id: id })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CategoryAssignmentRequest {
    pub category_id: Option<i64>,
}

/// `PUT /meditation/programs/{id}/category` — admin assign/clear
/// (spec FR-031).
pub async fn assign_category(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<CategoryAssignmentRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    SetProgramCategoryService::new(deps.programs.clone(), deps.categories.clone())
        .handle(SetProgramCategory {
            program_id: id,
            category_id: request.category_id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}
