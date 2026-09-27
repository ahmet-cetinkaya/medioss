//! HTTP surface for the meditation module: DTOs, extractors, routing.
//!
//! This crate never resolves sessions itself (web-api middleware inserts
//! the `Actor` into request extensions) and never touches a database; all
//! persistence flows through application ports, injected as trait objects
//! so the router stays concrete.

use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use meditation_application::ports::{
    AudioValidator, BlobStore, CategoryRepository, Clock, CoverValidator, MeditationRepository,
    MembershipRepository, NoteRepository, OpusConverter, PlaybackStateRepository,
    ProgramRepository, SessionRepository,
};

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

/// The meditation HTTP surface (plan D5). Every route is authenticated;
/// content-management routes are admin-gated through the
/// [`AdminActor`] extractor.
pub fn meditation_router() -> Router<SharedDependencies> {
    Router::new()
        // The extractor-level default body limit (2MB) would reject large
        // uploads before the handler's own streamed cap ever runs; the
        // upload route owns its limit (spec FR-002/003).
        .route(
            "/meditation/audio",
            post(crate::handlers_audio::upload).layer(axum::extract::DefaultBodyLimit::max(
                meditation_application::commands::MAX_UPLOAD_BYTES + 1024 * 1024,
            )),
        )
        .route("/meditation/library", get(crate::handlers_audio::library))
        .route(
            "/meditation/categories",
            get(crate::handlers_programs::categories)
                .post(crate::handlers_programs::create_category),
        )
        .route(
            "/meditation/categories/{id}",
            delete(crate::handlers_programs::delete_category),
        )
        .route(
            "/meditation/programs",
            get(crate::handlers_programs::programs)
                .post(crate::handlers_programs::create_program)
                .layer(axum::extract::DefaultBodyLimit::max(
                    meditation_application::commands::MAX_COVER_BYTES + 1024 * 1024,
                )),
        )
        .route(
            "/meditation/programs/{id}",
            get(crate::handlers_programs::program)
                .put(crate::handlers_programs::update_program)
                .delete(crate::handlers_programs::delete_program),
        )
        .route(
            "/meditation/programs/{id}/cover",
            get(crate::handlers_programs::cover)
                .put(crate::handlers_programs::replace_cover)
                .layer(axum::extract::DefaultBodyLimit::max(
                    meditation_application::commands::MAX_COVER_BYTES + 1024 * 1024,
                )),
        )
        .route(
            "/meditation/programs/{id}/members",
            post(crate::handlers_programs::add_member),
        )
        .route(
            "/meditation/programs/{id}/members/{meditation_id}",
            delete(crate::handlers_programs::remove_member),
        )
        .route(
            "/meditation/programs/{id}/category",
            put(crate::handlers_programs::assign_category),
        )
        .route(
            "/meditation/{id}",
            get(crate::handlers_audio::meditation).delete(crate::handlers_audio::delete_meditation),
        )
        .route("/meditation/{id}/audio", get(crate::handlers_audio::audio))
        .route(
            "/meditation/{id}/download",
            get(crate::handlers_audio::download),
        )
        .route(
            "/meditation/{id}/playback-state",
            get(crate::handlers_playback::playback_state).post(crate::handlers_playback::position),
        )
        .route(
            "/meditation/{id}/completion",
            post(crate::handlers_playback::completion),
        )
        .route(
            "/meditation/{id}/completed",
            put(crate::handlers_playback::set_completed),
        )
        .route(
            "/meditation/{id}/notes",
            get(crate::handlers_notes::notes).post(crate::handlers_notes::create_note),
        )
        .route(
            "/meditation/{id}/notes/{note_id}",
            put(crate::handlers_notes::update_note).delete(crate::handlers_notes::delete_note),
        )
}
