//! Per-caller program surfaces: the continue card and the recent strip.
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use meditation_application::queries::get_continue_program::GetContinueProgram;
use meditation_application::queries::list_recent_programs::ListRecentPrograms;
use serde::{Deserialize, Serialize};

use crate::programs::{program_response, progress_for, ProgramResponse};
use crate::{ApiError, CurrentActor, SharedDependencies};

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
    let found = deps
        .get_continue_program
        .handle(GetContinueProgram {
            user_id: actor.user_id(),
        })
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
    let listed = deps
        .list_recent_programs
        .handle(ListRecentPrograms {
            user_id: actor.user_id(),
            limit: query.limit,
        })
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

#[cfg(test)]
#[path = "program_activity.tests.rs"]
mod tests;
