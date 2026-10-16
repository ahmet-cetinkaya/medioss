//! HTTP surface for the meditation module: DTOs, extractors, routing.
//!
//! This crate never resolves sessions itself (web-api middleware inserts
//! the `Actor` into request extensions) and never touches a database; all
//! persistence flows through application ports, injected as trait objects
//! so the router stays concrete.

use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, post, put},
    Router,
};
use meditation_application::commands::{MAX_COVER_BYTES, MAX_UPLOAD_BYTES};
use meditation_application::ports::{
    AudioValidator, BlobStore, CategoryRepository, Clock, CoverValidator, MeditationRepository,
    MembershipRepository, NoteRepository, OpusConverter, PlaybackStateRepository,
    ProgramRepository, SessionRepository,
};

use crate::audio_stream::{audio, download};
use crate::audio_upload::upload;
use crate::categories::{
    assign_category, categories, create_category, delete_category, rename_category,
};
use crate::handlers_notes::{create_note, delete_note, notes, update_note};
use crate::handlers_playback::{completion, playback_state, position, set_completed};
use crate::handlers_statistics::statistics;
use crate::meditations::{
    daily_suggestion, delete_meditation, library, meditation, update_description,
};
use crate::program_activity::{continue_program, recent_programs};
use crate::program_cover::{cover, replace_cover};
use crate::program_members::{add_member, program_members, remove_member, update_order};
use crate::programs::{create_program, delete_program, program, programs, update_program};

/// Everything a handler needs, injected by web-api's composition root as
/// trait objects (all ports are object-safe through async_trait).
pub struct MeditationDependencies {
    pub meditations: Arc<dyn MeditationRepository>,
    pub programs: Arc<dyn ProgramRepository>,
    pub categories: Arc<dyn CategoryRepository>,
    pub memberships: Arc<dyn MembershipRepository>,
    pub notes: Arc<dyn NoteRepository>,
    pub playback: Arc<dyn PlaybackStateRepository>,
    pub sessions: Arc<dyn SessionRepository>,
    pub validator: Arc<dyn AudioValidator>,
    pub cover_validator: Arc<dyn CoverValidator>,
    pub converter: Arc<dyn OpusConverter>,
    pub store: Arc<dyn BlobStore>,
    pub clock: Arc<dyn Clock>,
    pub medioss_origin: Arc<String>,
}

pub type SharedDependencies = Arc<MeditationDependencies>;

/// Headroom above a payload cap for the multipart envelope around the file.
const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;

/// The meditation HTTP surface (plan D5). Every route is authenticated;
/// content-management routes are admin-gated through the
/// [`AdminActor`] extractor.
pub fn meditation_router() -> Router<SharedDependencies> {
    Router::new()
        .merge(audio_routes())
        .merge(category_routes())
        .merge(program_routes())
        .merge(program_member_routes())
        .merge(program_cover_routes())
        .merge(meditation_routes())
        .merge(playback_routes())
        .merge(note_routes())
}

fn audio_routes() -> Router<SharedDependencies> {
    Router::new()
        // The extractor-level default body limit (2MB) would reject large
        // uploads before the handler's own streamed cap ever runs; the
        // upload route owns its limit (spec FR-002/003).
        .route(
            "/meditation/audio",
            post(upload).layer(DefaultBodyLimit::max(
                MAX_UPLOAD_BYTES + MULTIPART_OVERHEAD_BYTES,
            )),
        )
        .route("/meditation/{id}/audio", get(audio))
        .route("/meditation/{id}/download", get(download))
}

fn category_routes() -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/meditation/categories",
            get(categories).post(create_category),
        )
        .route(
            "/meditation/categories/{id}",
            put(rename_category).delete(delete_category),
        )
        .route("/meditation/programs/{id}/category", put(assign_category))
}

fn program_routes() -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/meditation/programs",
            get(programs)
                .post(create_program)
                .layer(DefaultBodyLimit::max(
                    MAX_COVER_BYTES + MULTIPART_OVERHEAD_BYTES,
                )),
        )
        .route(
            "/meditation/programs/{id}",
            get(program).put(update_program).delete(delete_program),
        )
        .route("/meditation/continue", get(continue_program))
        .route("/meditation/recent-programs", get(recent_programs))
}

fn program_member_routes() -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/meditation/programs/{id}/members",
            get(program_members).post(add_member),
        )
        .route(
            "/meditation/programs/{id}/meditations/order",
            put(update_order),
        )
        .route(
            "/meditation/programs/{id}/members/{meditation_id}",
            delete(remove_member),
        )
}

fn program_cover_routes() -> Router<SharedDependencies> {
    Router::new().route(
        "/meditation/programs/{id}/cover",
        get(cover).put(replace_cover).layer(DefaultBodyLimit::max(
            MAX_COVER_BYTES + MULTIPART_OVERHEAD_BYTES,
        )),
    )
}

fn meditation_routes() -> Router<SharedDependencies> {
    Router::new()
        .route("/meditation/library", get(library))
        .route("/meditation/statistics", get(statistics))
        .route("/meditation/daily-suggestion", get(daily_suggestion))
        .route("/meditation/meditations/{id}", put(update_description))
        .route(
            "/meditation/{id}",
            get(meditation).delete(delete_meditation),
        )
}

fn playback_routes() -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/meditation/{id}/playback-state",
            get(playback_state).post(position),
        )
        .route("/meditation/{id}/completion", post(completion))
        .route("/meditation/{id}/completed", put(set_completed))
}

fn note_routes() -> Router<SharedDependencies> {
    Router::new()
        .route("/meditation/{id}/notes", get(notes).post(create_note))
        .route(
            "/meditation/{id}/notes/{note_id}",
            put(update_note).delete(delete_note),
        )
}
