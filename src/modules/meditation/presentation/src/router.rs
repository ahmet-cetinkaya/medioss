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
use tower_http::limit::RequestBodyLimitLayer;

use meditation_application::ports::{
    BlobStore, CoverValidator, PlaybackStateRepository, ProgramRepository, UploadStaging,
};

use crate::audio_stream::{audio, download};
use crate::audio_upload::upload;
use crate::categories::{
    assign_category, categories, create_category, delete_category, rename_category,
};
use crate::handlers_favorites::{favorite, favorites, unfavorite};
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
    pub create_category: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::create_category::CreateCategoryCommand, Output = meditation_application::ports::Category>>,
    pub rename_category: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::rename_category::RenameCategoryCommand, Output = ()>>,
    pub delete_category: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::delete_category::DeleteCategoryCommand, Output = ()>>,
    pub set_program_category: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::set_program_category::SetProgramCategoryCommand, Output = ()>>,
    pub list_categories: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_categories::ListCategoriesQuery, Output = Vec<meditation_application::ports::Category>>>,
    pub create_program: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::create_program::CreateProgramCommand, Output = meditation_application::ports::Program>>,
    pub update_program: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::update_program::UpdateProgramCommand, Output = ()>>,
    pub delete_program: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::delete_program::DeleteProgramCommand, Output = ()>>,
    pub list_programs: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_programs::ListProgramsQuery, Output = Vec<meditation_application::ports::Program>>>,
    pub list_program_members: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_program_members::ListProgramMembersQuery, Output = Vec<meditation_application::ports::Membership>>>,
    pub add_member: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::add_member::AddMemberCommand, Output = ()>>,
    pub remove_member: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::remove_member::RemoveMemberCommand, Output = ()>>,
    pub update_program_order: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::update_program_order::UpdateProgramOrderCommand, Output = ()>>,
    pub upload_meditation: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::upload_meditation::UploadMeditationCommand, Output = meditation_domain::Meditation>>,
    pub delete_meditation: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::delete_meditation::DeleteMeditationCommand, Output = ()>>,
    pub update_meditation_description: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionCommand, Output = meditation_domain::Meditation>>,
    pub get_meditation: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_meditation::GetMeditationQuery, Output = meditation_domain::Meditation>>,
    pub get_meditation_detail: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_meditation_detail::GetMeditationDetailQuery, Output = meditation_application::queries::get_meditation_detail::MeditationDetail>>,
    pub get_daily_suggestion: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_daily_suggestion::GetDailySuggestionQuery, Output = Option<meditation_application::queries::get_daily_suggestion::DailySuggestion>>>,
    pub list_library: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_library::ListLibraryQuery, Output = Vec<meditation_application::queries::list_library::LibraryEntry>>>,
    pub get_playback_state: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_playback_state::GetPlaybackStateQuery, Output = meditation_application::ports::PlaybackState>>,
    pub get_program_cover: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_program_cover::GetProgramCoverQuery, Output = Option<meditation_application::queries::get_program_cover::ProgramCover>>>,
    pub replace_program_cover: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::replace_program_cover::ReplaceProgramCoverCommand, Output = ()>>,
    pub record_position: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::record_position::RecordPositionCommand, Output = ()>>,
    pub record_completion: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::record_completion::RecordCompletionCommand, Output = meditation_application::ports::MeditationSession>>,
    pub set_completed: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::set_completed::SetCompletedCommand, Output = ()>>,
    pub get_user_statistics: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_user_statistics::GetUserStatisticsQuery, Output = meditation_application::queries::get_user_statistics::UserStatistics>>,
    pub create_note: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::create_note::CreateNoteCommand, Output = meditation_application::ports::Note>>,
    pub update_note: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::update_note::UpdateNoteCommand, Output = ()>>,
    pub delete_note: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::delete_note::DeleteNoteCommand, Output = ()>>,
    pub list_notes: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_notes::ListNotesQuery, Output = Vec<meditation_application::ports::Note>>>,
    pub favorite_meditation: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::favorite_meditation::FavoriteMeditationCommand, Output = ()>>,
    pub unfavorite_meditation: Arc<dyn meditation_application::ports::CommandHandler<meditation_application::commands::unfavorite_meditation::UnfavoriteMeditationCommand, Output = ()>>,
    pub list_favorite_meditations: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_favorite_meditations::ListFavoriteMeditationsQuery, Output = Vec<meditation_application::queries::list_favorite_meditations::FavoriteEntry>>>,
    pub list_favorite_ids: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_favorite_ids::ListFavoriteIdsQuery, Output = std::collections::HashSet<i64>>>,
    pub get_continue_program: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::get_continue_program::GetContinueProgramQuery, Output = Option<meditation_application::queries::get_continue_program::ContinueProgram>>>,
    pub list_recent_programs: Arc<dyn meditation_application::ports::QueryHandler<meditation_application::queries::list_recent_programs::ListRecentProgramsQuery, Output = Vec<meditation_application::queries::list_recent_programs::RecentProgram>>>,
    /// Program rows for the existence gates and the cover surfaces.
    pub programs: Arc<dyn ProgramRepository>,
    /// Home-program map decorating the library rows.
    /// Per-user progress read on every meditation surface.
    pub playback: Arc<dyn PlaybackStateRepository>,
    /// Cover normalization, run while streaming a replacement cover.
    pub cover_validator: Arc<dyn CoverValidator>,
    /// Blob reads and writes behind covers and audio streaming.
    pub store: Arc<dyn BlobStore>,
    /// Disk staging that audio uploads stream into instead of memory.
    pub staging: Arc<dyn UploadStaging>,
    /// The "now" feeding the statistics window.
    pub medioss_origin: Arc<String>,
    /// Cap on a meditation upload's audio field; `meditation_router` must receive the same value.
    pub max_upload_bytes: usize,
    /// Cap on a program cover; `meditation_router` must receive the same value.
    pub max_cover_bytes: usize,
}

pub type SharedDependencies = Arc<MeditationDependencies>;

/// Headroom above a payload cap for the multipart envelope around the file.
const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;

/// Body cap for a multipart upload route. `RequestBodyLimitLayer` answers 413
/// from a declared `Content-Length` over the cap before any body byte is read
/// (spec FR-002); `DefaultBodyLimit` raises axum's 2 MiB extractor default and
/// still counts bytes as they stream in (spec FR-003).
fn upload_body_limit(max_bytes: usize) -> (DefaultBodyLimit, RequestBodyLimitLayer) {
    let cap = max_bytes + MULTIPART_OVERHEAD_BYTES;
    (DefaultBodyLimit::max(cap), RequestBodyLimitLayer::new(cap))
}

/// The meditation HTTP surface (plan D5). Every route is authenticated;
/// content-management routes are admin-gated through the
/// [`AdminActor`] extractor.
pub fn meditation_router(
    max_upload_bytes: usize,
    max_cover_bytes: usize,
) -> Router<SharedDependencies> {
    Router::new()
        .merge(audio_routes(max_upload_bytes))
        .merge(category_routes())
        .merge(program_routes(max_cover_bytes))
        .merge(program_member_routes())
        .merge(program_cover_routes(max_cover_bytes))
        .merge(meditation_routes())
        .merge(playback_routes())
        .merge(note_routes())
        .merge(favorite_routes())
}

fn audio_routes(max_upload_bytes: usize) -> Router<SharedDependencies> {
    Router::new()
        // The extractor-level default body limit (2MB) would reject large
        // uploads before the handler's own streamed cap ever runs; the
        // upload route owns its limit (spec FR-002/003).
        .route(
            "/meditation/audio",
            post(upload).layer(upload_body_limit(max_upload_bytes)),
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

fn program_routes(max_cover_bytes: usize) -> Router<SharedDependencies> {
    Router::new()
        .route(
            "/meditation/programs",
            get(programs)
                .post(create_program)
                .layer(upload_body_limit(max_cover_bytes)),
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

fn program_cover_routes(max_cover_bytes: usize) -> Router<SharedDependencies> {
    Router::new().route(
        "/meditation/programs/{id}/cover",
        get(cover)
            .put(replace_cover)
            .layer(upload_body_limit(max_cover_bytes)),
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

fn favorite_routes() -> Router<SharedDependencies> {
    Router::new()
        .route("/meditation/favorites", get(favorites))
        .route(
            "/meditation/{id}/favorite",
            put(favorite).delete(unfavorite),
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
