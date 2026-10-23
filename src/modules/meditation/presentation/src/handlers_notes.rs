//! Note handlers: create, list, edit, delete — author-scoped throughout
//! (spec FR-034..037).
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use meditation_application::commands::create_note::{
    CreateNote, CreateNoteHandler, CreateNoteService,
};
use meditation_application::commands::delete_note::{
    DeleteNote, DeleteNoteHandler, DeleteNoteService,
};
use meditation_application::commands::update_note::{
    UpdateNote, UpdateNoteHandler, UpdateNoteService,
};
use meditation_application::queries::list_notes::{ListNotes, ListNotesHandler, ListNotesService};
use serde::{Deserialize, Serialize};

use crate::{ensure_browser_origin, ApiError, CurrentActor, SharedDependencies};

#[derive(Deserialize)]
pub struct NoteRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct NoteResponse {
    pub id: i64,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}

fn note_response(note: &meditation_application::ports::Note) -> NoteResponse {
    NoteResponse {
        id: note.id,
        text: note.text.clone(),
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    }
}

/// `GET /meditation/{id}/notes` — the caller's own notes, newest first
/// (spec FR-035).
pub async fn notes(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<NoteResponse>>, ApiError> {
    let listed = ListNotesService::new(deps.meditations.clone(), deps.notes.clone())
        .handle(ListNotes {
            user_id: actor.user_id(),
            meditation_id: id,
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(Json(listed.iter().map(note_response).collect()))
}

/// `POST /meditation/{id}/notes` — create (spec FR-034).
pub async fn create_note(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<NoteRequest>,
) -> Result<(StatusCode, Json<NoteResponse>), ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    let note = CreateNoteService::new(
        deps.meditations.clone(),
        deps.notes.clone(),
        deps.clock.clone(),
    )
    .handle(CreateNote {
        user_id: actor.user_id(),
        meditation_id: id,
        text: request.text.clone(),
    })
    .await
    .map_err(ApiError::Application)?;
    Ok((StatusCode::CREATED, Json(note_response(&note))))
}

/// `PUT /meditation/{id}/notes/{note_id}` — edit the author's note; another
/// author's id is indistinguishable from unknown (spec FR-036/037).
pub async fn update_note(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path((id, note_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    Json(request): Json<NoteRequest>,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    UpdateNoteService::new(deps.notes.clone(), deps.clock.clone())
        .handle(UpdateNote {
            note_id,
            meditation_id: id,
            user_id: actor.user_id(),
            text: request.text.clone(),
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /meditation/{id}/notes/{note_id}` — delete the author's note.
pub async fn delete_note(
    CurrentActor(actor): CurrentActor,
    State(deps): State<SharedDependencies>,
    Path((id, note_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    ensure_browser_origin(&headers, &deps.medioss_origin)?;
    DeleteNoteService::new(deps.notes.clone())
        .handle(DeleteNote {
            note_id,
            meditation_id: id,
            user_id: actor.user_id(),
        })
        .await
        .map_err(ApiError::Application)?;
    Ok(StatusCode::NO_CONTENT)
}
