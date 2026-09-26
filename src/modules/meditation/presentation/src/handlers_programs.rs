//! Program, category, and cover handlers (spec FR-025..033). Management
//! routes are admin-gated; reads are authenticated.
use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::{commands, queries};
use serde::{Deserialize, Serialize};

use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct ProgramResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub has_cover: bool,
    pub category_id: Option<i64>,
    pub created_at: String,
    /// The requesting user's completed share; `None` for empty programs
    /// (spec FR-029).
    pub progress: Option<f64>,
}

fn program_response(
    program: &meditation_application::ports::Program,
    progress: Option<f64>,
) -> ProgramResponse {
    ProgramResponse {
        id: program.id,
        title: program.title.clone(),
        description: program.description.clone(),
        has_cover: program.cover_storage_key.is_some(),
        category_id: program.category_id,
        created_at: program.created_at.to_rfc3339(),
        progress,
    }
}

/// Computes the caller's completed share over a program's members.
async fn progress_for(
    deps: &SharedDependencies,
    user_id: kernel::UserId,
    program_id: i64,
) -> Option<f64> {
    let members = queries::list_program_members(deps.memberships.as_ref(), program_id)
        .await
        .ok()?;
    if members.is_empty() {
        return None;
    }
    let mut completed = 0;
    for member in &members {
        if deps
            .playback
            .get(user_id, member.meditation_id)
            .await
            .ok()
            .flatten()
            .is_some_and(|state| state.completed)
        {
            completed += 1;
        }
    }
    Some(completed as f64 / members.len() as f64)
}

/// `GET /meditation/categories` — every category.
pub async fn categories(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let listed = queries::list_categories(deps.categories.as_ref())
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
    let category =
        commands::create_category(deps.categories.as_ref(), deps.clock.as_ref(), &request.name)
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

/// `DELETE /meditation/categories/{id}` — admin delete; programs become
/// uncategorized (spec FR-032).
pub async fn delete_category(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    commands::delete_category(deps.categories.as_ref(), deps.programs.as_ref(), id)
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /meditation/programs` — every program with the caller's progress.
pub async fn programs(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<ProgramResponse>>, ApiError> {
    let listed = queries::list_programs(deps.programs.as_ref())
        .await
        .map_err(ApiError::Application)?;
    let mut responses = Vec::with_capacity(listed.len());
    for program in &listed {
        let progress = progress_for(&deps, actor.user_id(), program.id).await;
        responses.push(program_response(program, progress));
    }
    Ok(Json(responses))
}

/// `GET /meditation/programs/{id}` — one program with the caller's progress.
pub async fn program(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<ProgramResponse>, ApiError> {
    let found = deps
        .programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;
    let progress = progress_for(&deps, actor.user_id(), id).await;
    Ok(Json(program_response(&found, progress)))
}

/// `POST /meditation/programs` — admin multipart create (title,
/// description, optional cover file) (spec FR-025/026).
pub async fn create_program(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ProgramResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;

    let max_cover = commands::MAX_COVER_BYTES;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut cover: Option<Vec<u8>> = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?
    {
        match field.name() {
            Some("title") => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?,
                );
            }
            Some("description") => {
                description = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?,
                );
            }
            Some("cover") => {
                let mut buffer = Vec::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    buffer.extend_from_slice(&chunk);
                    if buffer.len() > max_cover {
                        return Err(ApiError::StatusCode(StatusCode::PAYLOAD_TOO_LARGE));
                    }
                }
                cover = Some(buffer);
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let title = title.ok_or(ApiError::StatusCode(StatusCode::BAD_REQUEST))?;
    let program = commands::create_program(
        deps.programs.as_ref(),
        deps.cover_validator.as_ref(),
        deps.store.as_ref(),
        deps.clock.as_ref(),
        &title,
        description.as_deref(),
        cover,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok((StatusCode::CREATED, Json(program_response(&program, None))))
}

#[derive(Deserialize)]
pub struct UpdateProgramRequest {
    pub title: String,
    pub description: Option<String>,
}

/// `PUT /meditation/programs/{id}` — admin update.
pub async fn update_program(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<UpdateProgramRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    commands::update_program(
        deps.programs.as_ref(),
        id,
        &request.title,
        request.description.as_deref(),
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /meditation/programs/{id}` — admin delete; meditations stay
/// (spec FR-028).
pub async fn delete_program(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    commands::delete_program(
        deps.programs.as_ref(),
        deps.memberships.as_ref(),
        deps.store.as_ref(),
        id,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /meditation/programs/{id}/cover` — the cover image, authenticated
/// (spec FR-026).
pub async fn cover(
    CurrentActor(_actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let program = deps
        .programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;
    let key = program.cover_storage_key.ok_or(ApiError::Application(
        meditation_application::ApplicationError::NotFound,
    ))?;
    let bytes = deps
        .store
        .read(&key)
        .await
        .map_err(meditation_application::ApplicationError::Storage)
        .map_err(ApiError::Application)?;
    let content_type = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else {
        "image/webp"
    };
    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(content_type),
        )],
        bytes,
    )
        .into_response())
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
    commands::add_member(
        deps.meditations.as_ref(),
        deps.programs.as_ref(),
        deps.memberships.as_ref(),
        deps.clock.as_ref(),
        id,
        request.meditation_id,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(StatusCode::CREATED)
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
    commands::remove_member(
        deps.programs.as_ref(),
        deps.memberships.as_ref(),
        id,
        meditation_id,
    )
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
    commands::set_program_category(
        deps.programs.as_ref(),
        deps.categories.as_ref(),
        id,
        request.category_id,
    )
    .await
    .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}
