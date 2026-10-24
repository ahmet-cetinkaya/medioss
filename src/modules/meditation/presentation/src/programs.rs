//! Program handlers (spec FR-025..028): list, read, create, update, delete.
//! Management routes are admin-gated; reads are authenticated.
use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands;
use meditation_application::commands::create_program::CreateProgram;
use meditation_application::commands::delete_program::DeleteProgram;
use meditation_application::commands::update_program::UpdateProgram;
use meditation_application::queries::list_program_members::ListProgramMembers;
use meditation_application::queries::list_programs::ListPrograms;
use serde::{Deserialize, Serialize};

use crate::response::read_field_capped;
use crate::{ensure_browser_origin, AdminActor, ApiError, CurrentActor, SharedDependencies};

// Descriptions stay modest: ten times the name bound.
const DESCRIPTION_TO_NAME_LIMIT_RATIO: usize = 10;

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

pub(crate) fn program_response(
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
pub(crate) async fn progress_for(
    deps: &SharedDependencies,
    user_id: kernel::UserId,
    program_id: i64,
) -> f64 {
    let Ok(members) = deps
        .list_program_members
        .handle(ListProgramMembers { program_id })
        .await
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

/// `GET /meditation/programs` — every program with the caller's progress.
pub async fn programs(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
) -> Result<Json<Vec<ProgramResponse>>, ApiError> {
    let listed = deps
        .list_programs
        .handle(ListPrograms)
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
                let text = read_field_capped(&mut field, max_text).await?;
                title = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("description") => {
                let text =
                    read_field_capped(&mut field, max_text * DESCRIPTION_TO_NAME_LIMIT_RATIO)
                        .await?;
                description = Some(String::from_utf8_lossy(&text).into_owned());
            }
            Some("cover") => {
                cover = Some(read_field_capped(&mut field, max_cover).await?);
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    let title = title.ok_or(ApiError::StatusCode(StatusCode::BAD_REQUEST))?;
    let program = deps
        .create_program
        .handle(CreateProgram {
            title,
            description,
            cover_bytes: cover,
        })
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
    deps.update_program
        .handle(UpdateProgram {
            program_id: id,
            title: request.title,
            description: request.description,
        })
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
    deps.delete_program
        .handle(DeleteProgram { program_id: id })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}
