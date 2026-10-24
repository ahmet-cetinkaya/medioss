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
use meditation_application::commands::add_member::AddMemberHandler;
use meditation_application::commands::create_category::CreateCategoryHandler;
use meditation_application::commands::create_note::CreateNoteHandler;
use meditation_application::commands::create_program::CreateProgramHandler;
use meditation_application::commands::delete_category::DeleteCategoryHandler;
use meditation_application::commands::delete_meditation::DeleteMeditationHandler;
use meditation_application::commands::delete_note::DeleteNoteHandler;
use meditation_application::commands::delete_program::DeleteProgramHandler;
use meditation_application::commands::record_completion::RecordCompletionHandler;
use meditation_application::commands::record_position::RecordPositionHandler;
use meditation_application::commands::remove_member::RemoveMemberHandler;
use meditation_application::commands::rename_category::RenameCategoryHandler;
use meditation_application::commands::set_completed::SetCompletedHandler;
use meditation_application::commands::set_program_category::SetProgramCategoryHandler;
use meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionHandler;
use meditation_application::commands::update_note::UpdateNoteHandler;
use meditation_application::commands::update_program::UpdateProgramHandler;
use meditation_application::commands::update_program_order::UpdateProgramOrderHandler;
use meditation_application::commands::upload_meditation::UploadMeditationHandler;
use meditation_application::commands::{MAX_COVER_BYTES, MAX_UPLOAD_BYTES};
use meditation_application::ports::{
    BlobStore, Clock, CoverValidator, MembershipRepository, PlaybackStateRepository,
    ProgramRepository,
};
use meditation_application::queries::get_continue_program::GetContinueProgramHandler;
use meditation_application::queries::get_daily_suggestion::GetDailySuggestionHandler;
use meditation_application::queries::get_meditation::GetMeditationHandler;
use meditation_application::queries::get_meditation_detail::GetMeditationDetailHandler;
use meditation_application::queries::get_playback_state::GetPlaybackStateHandler;
use meditation_application::queries::get_user_statistics::GetUserStatisticsHandler;
use meditation_application::queries::list_categories::ListCategoriesHandler;
use meditation_application::queries::list_library::ListLibraryHandler;
use meditation_application::queries::list_notes::ListNotesHandler;
use meditation_application::queries::list_program_members::ListProgramMembersHandler;
use meditation_application::queries::list_programs::ListProgramsHandler;
use meditation_application::queries::list_recent_programs::ListRecentProgramsHandler;

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

/// Everything a handler needs, injected by web-api's composition root: one
/// use-case handler per route (web-api builds the services from the
/// repositories), plus the ports presentation reads directly — program
/// existence gates and cover rows, the library's home-program map, per-user
/// playback decoration, cover normalization, blob reads for streaming, and
/// the statistics clock. The audio validator and Opus converter stay
/// web-api-side: only the upload service consumes them.
pub struct MeditationDependencies {
    pub create_category: Arc<dyn CreateCategoryHandler>,
    pub rename_category: Arc<dyn RenameCategoryHandler>,
    pub delete_category: Arc<dyn DeleteCategoryHandler>,
    pub set_program_category: Arc<dyn SetProgramCategoryHandler>,
    pub list_categories: Arc<dyn ListCategoriesHandler>,
    pub create_program: Arc<dyn CreateProgramHandler>,
    pub update_program: Arc<dyn UpdateProgramHandler>,
    pub delete_program: Arc<dyn DeleteProgramHandler>,
    pub list_programs: Arc<dyn ListProgramsHandler>,
    pub list_program_members: Arc<dyn ListProgramMembersHandler>,
    pub add_member: Arc<dyn AddMemberHandler>,
    pub remove_member: Arc<dyn RemoveMemberHandler>,
    pub update_program_order: Arc<dyn UpdateProgramOrderHandler>,
    pub upload_meditation: Arc<dyn UploadMeditationHandler>,
    pub delete_meditation: Arc<dyn DeleteMeditationHandler>,
    pub update_meditation_description: Arc<dyn UpdateMeditationDescriptionHandler>,
    pub get_meditation: Arc<dyn GetMeditationHandler>,
    pub get_meditation_detail: Arc<dyn GetMeditationDetailHandler>,
    pub get_daily_suggestion: Arc<dyn GetDailySuggestionHandler>,
    pub list_library: Arc<dyn ListLibraryHandler>,
    pub get_playback_state: Arc<dyn GetPlaybackStateHandler>,
    pub record_position: Arc<dyn RecordPositionHandler>,
    pub record_completion: Arc<dyn RecordCompletionHandler>,
    pub set_completed: Arc<dyn SetCompletedHandler>,
    pub get_user_statistics: Arc<dyn GetUserStatisticsHandler>,
    pub create_note: Arc<dyn CreateNoteHandler>,
    pub update_note: Arc<dyn UpdateNoteHandler>,
    pub delete_note: Arc<dyn DeleteNoteHandler>,
    pub list_notes: Arc<dyn ListNotesHandler>,
    pub get_continue_program: Arc<dyn GetContinueProgramHandler>,
    pub list_recent_programs: Arc<dyn ListRecentProgramsHandler>,
    /// Program rows for the existence gates and the cover surfaces.
    pub programs: Arc<dyn ProgramRepository>,
    /// Home-program map decorating the library rows.
    pub memberships: Arc<dyn MembershipRepository>,
    /// Per-user progress read on every meditation surface.
    pub playback: Arc<dyn PlaybackStateRepository>,
    /// Cover normalization, run while streaming a replacement cover.
    pub cover_validator: Arc<dyn CoverValidator>,
    /// Blob reads and writes behind covers and audio streaming.
    pub store: Arc<dyn BlobStore>,
    /// The "now" feeding the statistics window.
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
