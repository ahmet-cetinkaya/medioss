//! Meditation wiring in the composition root: repositories over the shared
//! SQLite connection, the media-store-backed blob store, and the ffmpeg
//! adapters.

use std::sync::Arc;

use meditation_application::commands::add_member::AddMemberService;
use meditation_application::commands::create_category::CreateCategoryService;
use meditation_application::commands::create_note::CreateNoteService;
use meditation_application::commands::create_program::CreateProgramService;
use meditation_application::commands::delete_category::DeleteCategoryService;
use meditation_application::commands::delete_meditation::DeleteMeditationService;
use meditation_application::commands::delete_note::DeleteNoteService;
use meditation_application::commands::delete_program::DeleteProgramService;
use meditation_application::commands::record_completion::RecordCompletionService;
use meditation_application::commands::record_position::RecordPositionService;
use meditation_application::commands::remove_member::RemoveMemberService;
use meditation_application::commands::rename_category::RenameCategoryService;
use meditation_application::commands::replace_program_cover::ReplaceProgramCoverService;
use meditation_application::commands::set_completed::SetCompletedService;
use meditation_application::commands::set_program_category::SetProgramCategoryService;
use meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionService;
use meditation_application::commands::update_note::UpdateNoteService;
use meditation_application::commands::update_program::UpdateProgramService;
use meditation_application::commands::update_program_order::UpdateProgramOrderService;
use meditation_application::commands::upload_meditation::UploadMeditationService;
use meditation_application::ports::{NoteRepository, PlaybackStateRepository, SessionRepository};
use meditation_application::queries::get_continue_program::GetContinueProgramService;
use meditation_application::queries::get_daily_suggestion::GetDailySuggestionService;
use meditation_application::queries::get_meditation::GetMeditationService;
use meditation_application::queries::get_meditation_detail::GetMeditationDetailService;
use meditation_application::queries::get_playback_state::GetPlaybackStateService;
use meditation_application::queries::get_program_cover::GetProgramCoverService;
use meditation_application::queries::get_user_statistics::GetUserStatisticsService;
use meditation_application::queries::list_categories::ListCategoriesService;
use meditation_application::queries::list_library::ListLibraryService;
use meditation_application::queries::list_notes::ListNotesService;
use meditation_application::queries::list_program_members::ListProgramMembersService;
use meditation_application::queries::list_programs::ListProgramsService;
use meditation_application::queries::list_recent_programs::ListRecentProgramsService;
use meditation_infrastructure::media::{FfmpegAudioValidator, FfmpegOpusConverter};
use meditation_infrastructure::persistence::{
    SeaOrmCategoryRepository, SeaOrmMeditationRepository, SeaOrmMembershipRepository,
    SeaOrmNoteRepository, SeaOrmPlaybackRepository, SeaOrmProgramRepository,
    SeaOrmSessionRepository, SystemClock as MeditationClock,
};
use meditation_presentation::{MeditationDependencies, SharedDependencies as MeditationShared};
use sea_orm::DatabaseConnection;

/// Implements the application BlobStore port over the shared media-store
/// crate. Keys are folder/file relative paths owned by the store.
pub struct MediaStoreBlobs {
    store: media_store::MediaStore,
}

impl MediaStoreBlobs {
    /// Builds an adapter over the media root.
    pub fn new(store: media_store::MediaStore) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl meditation_application::ports::BlobStore for MediaStoreBlobs {
    async fn put(
        &self,
        folder: &str,
        stem: &str,
        ext: &str,
        bytes: &[u8],
    ) -> Result<meditation_application::ports::StoredBlob, String> {
        let key = self
            .store
            .put(folder, stem, ext, bytes)
            .await
            .map_err(|error| error.to_string())?;
        Ok(meditation_application::ports::StoredBlob {
            storage_key: key.as_str().to_owned(),
            size_bytes: bytes.len() as i64,
        })
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        let key = media_store::StorageKey::parse_with_separator(key)
            .map_err(|error| error.to_string())?;
        self.store
            .delete(&key)
            .await
            .map_err(|error| error.to_string())
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, String> {
        let key = media_store::StorageKey::parse_with_separator(key)
            .map_err(|error| error.to_string())?;
        let stream = self
            .store
            .open(&key)
            .await
            .map_err(|error| error.to_string())?;
        let mut reader = stream.reader();
        let mut buffer = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut buffer)
            .await
            .map_err(|error| error.to_string())?;
        Ok(buffer)
    }

    async fn relocate(
        &self,
        key: &str,
        folder: &str,
        stem: &str,
        ext: &str,
    ) -> Result<String, String> {
        let key = media_store::StorageKey::parse_with_separator(key)
            .map_err(|error| error.to_string())?;
        let moved = self
            .store
            .rename(&key, folder, stem, ext)
            .await
            .map_err(|error| error.to_string())?;
        Ok(moved.as_str().to_owned())
    }

    async fn delete_folder(&self, folder: &str) -> Result<(), String> {
        self.store
            .delete_folder(folder)
            .await
            .map_err(|error| error.to_string())
    }
}

/// True when the ffmpeg binary answers `-version`: the converter depends on
/// it, and a missing encoder must be an operator-visible warning, not a
/// per-upload surprise (plan D2).
pub fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Builds the meditation dependencies over the shared connection and data
/// dir, returning them together with the note/playback/session ports that
/// user deletion purges directly. A missing ffmpeg logs a warning; uploads
/// surface 503 until the operator installs it.
/// The module dependencies plus the three raw ports that user deletion
/// purges directly (notes, playback states, sessions).
pub struct MeditationModule {
    pub shared: MeditationShared,
    pub notes: Arc<dyn NoteRepository>,
    pub playback: Arc<dyn PlaybackStateRepository>,
    pub sessions: Arc<dyn SessionRepository>,
}

pub fn meditation_dependencies(
    connection: DatabaseConnection,
    data_dir: &std::path::Path,
    medioss_origin: Arc<String>,
) -> MeditationModule {
    if !ffmpeg_available() {
        tracing::warn!(
            "ffmpeg not found on PATH: meditation uploads will fail with 503 until it is installed"
        );
    }
    let media_root = data_dir.join("media").join("meditation");
    let notes: Arc<dyn NoteRepository> = Arc::new(SeaOrmNoteRepository::new(connection.clone()));
    let playback: Arc<dyn PlaybackStateRepository> =
        Arc::new(SeaOrmPlaybackRepository::new(connection.clone()));
    let sessions: Arc<dyn SessionRepository> =
        Arc::new(SeaOrmSessionRepository::new(connection.clone()));
    let dependencies = meditation_dependencies_with_ports(
        Arc::new(SeaOrmMeditationRepository::new(connection.clone())),
        Arc::new(SeaOrmProgramRepository::new(connection.clone())),
        Arc::new(SeaOrmCategoryRepository::new(connection.clone())),
        Arc::new(SeaOrmMembershipRepository::new(connection)),
        Arc::clone(&notes),
        Arc::clone(&playback),
        Arc::clone(&sessions),
        Arc::new(FfmpegAudioValidator),
        Arc::new(CoverNormalizer),
        Arc::new(FfmpegOpusConverter),
        Arc::new(MediaStoreBlobs::new(media_store::MediaStore::new(
            media_root,
        ))),
        Arc::new(MeditationClock),
        medioss_origin,
    );
    MeditationModule {
        shared: dependencies,
        notes,
        playback,
        sessions,
    }
}

/// Assembles every use-case service over the given ports. Split out from
/// [`meditation_dependencies`] so tests and other call sites can share the
/// note/playback/session ports with user deletion without going through the
/// SeaORM constructors again.
#[allow(clippy::too_many_arguments)]
fn meditation_dependencies_with_ports(
    meditations: Arc<dyn meditation_application::ports::MeditationRepository>,
    programs: Arc<dyn meditation_application::ports::ProgramRepository>,
    categories: Arc<dyn meditation_application::ports::CategoryRepository>,
    memberships: Arc<dyn meditation_application::ports::MembershipRepository>,
    notes: Arc<dyn NoteRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    sessions: Arc<dyn SessionRepository>,
    validator: Arc<dyn meditation_application::ports::AudioValidator>,
    cover_validator: Arc<dyn meditation_application::ports::CoverValidator>,
    converter: Arc<dyn meditation_application::ports::OpusConverter>,
    store: Arc<dyn meditation_application::ports::BlobStore>,
    clock: Arc<dyn meditation_application::ports::Clock>,
    medioss_origin: Arc<String>,
) -> MeditationShared {
    Arc::new(MeditationDependencies {
        create_category: Arc::new(CreateCategoryService::new(
            categories.clone(),
            clock.clone(),
        )),
        rename_category: Arc::new(RenameCategoryService::new(
            categories.clone(),
            clock.clone(),
        )),
        delete_category: Arc::new(DeleteCategoryService::new(
            categories.clone(),
            programs.clone(),
        )),
        set_program_category: Arc::new(SetProgramCategoryService::new(
            programs.clone(),
            categories.clone(),
        )),
        list_categories: Arc::new(ListCategoriesService::new(categories.clone())),
        create_program: Arc::new(CreateProgramService::new(
            programs.clone(),
            cover_validator.clone(),
            store.clone(),
            clock.clone(),
        )),
        update_program: Arc::new(UpdateProgramService::new(programs.clone())),
        delete_program: Arc::new(DeleteProgramService::new(
            programs.clone(),
            memberships.clone(),
            meditations.clone(),
            store.clone(),
        )),
        list_programs: Arc::new(ListProgramsService::new(programs.clone())),
        list_program_members: Arc::new(ListProgramMembersService::new(
            programs.clone(),
            memberships.clone(),
        )),
        add_member: Arc::new(AddMemberService::new(
            meditations.clone(),
            programs.clone(),
            memberships.clone(),
            clock.clone(),
        )),
        remove_member: Arc::new(RemoveMemberService::new(
            programs.clone(),
            memberships.clone(),
        )),
        update_program_order: Arc::new(UpdateProgramOrderService::new(
            programs.clone(),
            memberships.clone(),
        )),
        upload_meditation: Arc::new(UploadMeditationService::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            validator,
            converter,
            store.clone(),
            clock.clone(),
        )),
        delete_meditation: Arc::new(DeleteMeditationService::new(
            meditations.clone(),
            memberships.clone(),
            notes.clone(),
            store.clone(),
        )),
        update_meditation_description: Arc::new(UpdateMeditationDescriptionService::new(
            meditations.clone(),
        )),
        get_meditation: Arc::new(GetMeditationService::new(meditations.clone())),
        get_meditation_detail: Arc::new(GetMeditationDetailService::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        get_daily_suggestion: Arc::new(GetDailySuggestionService::new(
            meditations.clone(),
            clock.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_library: Arc::new(ListLibraryService::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            categories,
        )),
        get_program_cover: Arc::new(GetProgramCoverService::new(programs.clone(), store.clone())),
        replace_program_cover: Arc::new(ReplaceProgramCoverService::new(
            programs.clone(),
            store.clone(),
        )),
        get_playback_state: Arc::new(GetPlaybackStateService::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_position: Arc::new(RecordPositionService::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_completion: Arc::new(RecordCompletionService::new(
            meditations.clone(),
            playback.clone(),
            sessions.clone(),
        )),
        set_completed: Arc::new(SetCompletedService::new(
            meditations.clone(),
            playback.clone(),
        )),
        get_user_statistics: Arc::new(GetUserStatisticsService::new(
            sessions.clone(),
            clock.clone(),
        )),
        create_note: Arc::new(CreateNoteService::new(
            meditations.clone(),
            notes.clone(),
            clock.clone(),
        )),
        update_note: Arc::new(UpdateNoteService::new(notes.clone(), clock.clone())),
        delete_note: Arc::new(DeleteNoteService::new(notes.clone())),
        list_notes: Arc::new(ListNotesService::new(meditations, notes)),
        get_continue_program: Arc::new(GetContinueProgramService::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_recent_programs: Arc::new(ListRecentProgramsService::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        programs,
        playback,
        cover_validator,
        store,
        medioss_origin,
    })
}

const MAX_COVER_DIMENSION: u32 = 1_024;
const MAX_COVER_OUTPUT_BYTES: usize = meditation_application::commands::MAX_COVER_BYTES;

struct CoverNormalizer;

#[async_trait::async_trait]
impl meditation_application::ports::CoverValidator for CoverNormalizer {
    async fn normalize(
        &self,
        bytes: Vec<u8>,
    ) -> Result<Vec<u8>, meditation_application::ports::CoverNormalizationError> {
        crate::image_normalization::normalize(
            bytes,
            crate::image_normalization::ImageSpec {
                maximum_dimension: MAX_COVER_DIMENSION,
                maximum_output_bytes: MAX_COVER_OUTPUT_BYTES,
                square: false,
            },
        )
        .await
        .map_err(meditation_application::ports::CoverNormalizationError::from)
    }
}
