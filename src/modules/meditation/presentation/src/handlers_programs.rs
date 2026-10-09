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

#[derive(Deserialize)]
pub struct RenameCategoryRequest {
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
    /// The requesting user's completed share; every program surface must
    /// show it, so an empty program reports `0.0` rather than omitting it.
    pub progress: f64,
}

fn program_response(
    program: &meditation_application::ports::Program,
    progress: f64,
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

/// Computes the caller's completed share over a program's members; `0.0`
/// when the program has no members yet.
async fn progress_for(deps: &SharedDependencies, user_id: kernel::UserId, program_id: i64) -> f64 {
    let Ok(members) = queries::list_program_members(deps.memberships.as_ref(), program_id).await
    else {
        return 0.0;
    };
    if members.is_empty() {
        return 0.0;
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
    completed as f64 / members.len() as f64
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
    commands::rename_category(
        deps.categories.as_ref(),
        deps.clock.as_ref(),
        id,
        &request.name,
    )
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

/// `GET /meditation/programs/{id}/members` — the program's meditations, in
/// program order, each with the caller's own progress.
pub async fn program_members(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<crate::handlers_audio::MeditationResponse>>, ApiError> {
    deps.programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;
    let members = queries::list_program_members(deps.memberships.as_ref(), id)
        .await
        .map_err(ApiError::Application)?;
    let mut responses = Vec::with_capacity(members.len());
    for member in &members {
        let meditation = queries::get_meditation(deps.meditations.as_ref(), member.meditation_id)
            .await
            .map_err(ApiError::Application)?;
        responses.push(
            crate::handlers_audio::with_playback(
                deps.playback.as_ref(),
                actor.user_id(),
                &meditation,
            )
            .await,
        );
    }
    Ok(Json(responses))
}

#[derive(Serialize)]
pub struct ContinueProgramResponse {
    pub program: ProgramResponse,
    pub meditation_id: i64,
}

/// `GET /meditation/continue` — the program holding the caller's most
/// recently touched meditation, or `204` when there is none yet.
pub async fn continue_program(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Response, ApiError> {
    let found = queries::get_continue_program(
        deps.playback.as_ref(),
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
        actor.user_id(),
    )
    .await
    .map_err(ApiError::Application)?;
    let Some(found) = found else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };
    let progress = progress_for(&deps, actor.user_id(), found.program.id).await;
    Ok(Json(ContinueProgramResponse {
        program: program_response(&found.program, progress),
        meditation_id: found.meditation_id,
    })
    .into_response())
}

#[derive(Deserialize)]
pub struct RecentProgramsQuery {
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct RecentProgramResponse {
    pub program: ProgramResponse,
    pub last_touched_at: String,
}

/// `GET /meditation/recent-programs?limit=N` — the caller's touched
/// programs, most recently touched first; `limit` caps the count (the home
/// page's strip asks for 5, the history page omits it for the full list).
pub async fn recent_programs(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    axum::extract::Query(query): axum::extract::Query<RecentProgramsQuery>,
) -> Result<Json<Vec<RecentProgramResponse>>, ApiError> {
    let listed = queries::list_recent_programs(
        deps.playback.as_ref(),
        deps.memberships.as_ref(),
        deps.programs.as_ref(),
        actor.user_id(),
        query.limit,
    )
    .await
    .map_err(ApiError::Application)?;
    let mut responses = Vec::with_capacity(listed.len());
    for entry in &listed {
        let progress = progress_for(&deps, actor.user_id(), entry.program.id).await;
        responses.push(RecentProgramResponse {
            program: program_response(&entry.program, progress),
            last_touched_at: entry.last_touched_at.to_rfc3339(),
        });
    }
    Ok(Json(responses))
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
    let max_text = commands::MAX_NAME_LENGTH * 4;
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
                let text = crate::response::read_field_capped(&mut field, max_text).await?;
                title = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("description") => {
                // Descriptions stay modest: ten times the name bound.
                let text = crate::response::read_field_capped(&mut field, max_text * 10).await?;
                description = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("cover") => {
                cover = Some(crate::response::read_field_capped(&mut field, max_cover).await?);
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
    Ok((StatusCode::CREATED, Json(program_response(&program, 0.0))))
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
        deps.meditations.as_ref(),
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

/// `PUT /meditation/programs/{id}/cover` (spec FR-026 follow-up: covers are
/// replaceable, not create-only). Removes the previous cover blob, if any.
pub async fn replace_cover(
    AdminActor(_actor): AdminActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;

    let max_cover = commands::MAX_COVER_BYTES;
    let mut cover: Option<(Vec<u8>, &'static str)> = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::StatusCode(StatusCode::BAD_REQUEST))?
    {
        if field.name() == Some("cover") {
            let buffer = crate::response::read_field_capped(&mut field, max_cover).await?;
            let format = deps.cover_validator.sniff(&buffer).await.map_err(|_| {
                ApiError::Application(meditation_application::ApplicationError::UnsupportedMedia)
            })?;
            cover = Some((buffer, format.extension()));
        } else {
            let _ = field.bytes().await;
        }
    }
    let Some((bytes, ext)) = cover else {
        return Err(ApiError::StatusCode(StatusCode::BAD_REQUEST));
    };

    let program = deps
        .programs
        .find_by_id(id)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?
        .ok_or(ApiError::Application(
            meditation_application::ApplicationError::NotFound,
        ))?;

    let blob = deps
        .store
        .put(&program.folder, "cover", ext, &bytes)
        .await
        .map_err(meditation_application::ApplicationError::Storage)
        .map_err(ApiError::Application)?;
    deps.programs
        .set_cover_key(id, &blob.storage_key)
        .await
        .map_err(meditation_application::ApplicationError::Repository)
        .map_err(ApiError::Application)?;
    if let Some(old_key) = program.cover_storage_key {
        if old_key != blob.storage_key {
            let _ = deps.store.delete(&old_key).await;
        }
    }
    Ok(StatusCode::NO_CONTENT)
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
    commands::update_program_order(
        deps.programs.as_ref(),
        deps.memberships.as_ref(),
        id,
        &request.meditation_ids,
    )
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
