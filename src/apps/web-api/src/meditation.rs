//! Meditation wiring in the composition root: repositories over the shared
//! SQLite connection, the media-store-backed blob store, and the ffmpeg
//! adapters.

use std::sync::Arc;

use crate::config::UploadLimits;

use meditation_application::commands::add_member::AddMemberCommandHandler;
use meditation_application::commands::create_category::CreateCategoryCommandHandler;
use meditation_application::commands::create_note::CreateNoteCommandHandler;
use meditation_application::commands::create_program::CreateProgramCommandHandler;
use meditation_application::commands::delete_category::DeleteCategoryCommandHandler;
use meditation_application::commands::delete_meditation::DeleteMeditationCommandHandler;
use meditation_application::commands::delete_note::DeleteNoteCommandHandler;
use meditation_application::commands::delete_program::DeleteProgramCommandHandler;
use meditation_application::commands::favorite_meditation::FavoriteMeditationCommandHandler;
use meditation_application::commands::record_completion::RecordCompletionCommandHandler;
use meditation_application::commands::record_position::RecordPositionCommandHandler;
use meditation_application::commands::remove_member::RemoveMemberCommandHandler;
use meditation_application::commands::rename_category::RenameCategoryCommandHandler;
use meditation_application::commands::replace_program_cover::ReplaceProgramCoverCommandHandler;
use meditation_application::commands::set_completed::SetCompletedCommandHandler;
use meditation_application::commands::set_program_category::SetProgramCategoryCommandHandler;
use meditation_application::commands::unfavorite_meditation::UnfavoriteMeditationCommandHandler;
use meditation_application::commands::update_meditation_description::UpdateMeditationDescriptionCommandHandler;
use meditation_application::commands::update_note::UpdateNoteCommandHandler;
use meditation_application::commands::update_program::UpdateProgramCommandHandler;
use meditation_application::commands::update_program_order::UpdateProgramOrderCommandHandler;
use meditation_application::commands::upload_meditation::UploadMeditationCommandHandler;
use meditation_application::ports::{
    FavoriteRepository, NoteRepository, PlaybackStateRepository, SessionRepository,
};
use meditation_application::queries::get_continue_program::GetContinueProgramQueryHandler;
use meditation_application::queries::get_daily_suggestion::GetDailySuggestionQueryHandler;
use meditation_application::queries::get_meditation::GetMeditationQueryHandler;
use meditation_application::queries::get_meditation_detail::GetMeditationDetailQueryHandler;
use meditation_application::queries::get_playback_state::GetPlaybackStateQueryHandler;
use meditation_application::queries::get_program_cover::GetProgramCoverQueryHandler;
use meditation_application::queries::get_user_statistics::GetUserStatisticsQueryHandler;
use meditation_application::queries::list_categories::ListCategoriesQueryHandler;
use meditation_application::queries::list_favorite_ids::ListFavoriteIdsQueryHandler;
use meditation_application::queries::list_favorite_meditations::ListFavoriteMeditationsQueryHandler;
use meditation_application::queries::list_library::ListLibraryQueryHandler;
use meditation_application::queries::list_notes::ListNotesQueryHandler;
use meditation_application::queries::list_program_members::ListProgramMembersQueryHandler;
use meditation_application::queries::list_programs::ListProgramsQueryHandler;
use meditation_application::queries::list_recent_programs::ListRecentProgramsQueryHandler;
use meditation_infrastructure::media::{
    FfmpegAudioValidator, FfmpegOpusConverter, TempFileStaging,
};
use meditation_infrastructure::persistence::{
    SeaOrmCategoryRepository, SeaOrmFavoriteRepository, SeaOrmMeditationRepository,
    SeaOrmMembershipRepository, SeaOrmNoteRepository, SeaOrmPlaybackRepository,
    SeaOrmProgramRepository, SeaOrmSessionRepository, SystemClock as MeditationClock,
};
use meditation_presentation::{MeditationDependencies, SharedDependencies as MeditationShared};
use sea_orm::DatabaseConnection;

/// Implements the application BlobStore port over the shared media-store
/// crate. Keys are folder/file relative paths owned by the store.
pub struct MediaStoreBlobs {
    store: media_store::MediaStore,
    staging_root: std::path::PathBuf,
}

impl MediaStoreBlobs {
    /// Builds an adapter over the media root; staged files named by
    /// `put_staged` live in `staging_root`.
    pub fn new(store: media_store::MediaStore, staging_root: std::path::PathBuf) -> Self {
        Self {
            store,
            staging_root,
        }
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

    async fn put_staged(
        &self,
        media: meditation_application::ports::StagedMedia,
        folder: &str,
        stem: &str,
        ext: &str,
    ) -> Result<meditation_application::ports::StoredBlob, String> {
        let staged_path = meditation_infrastructure::media::resolve_staged_path(
            &self.staging_root,
            &media.address,
        )
        .map_err(|error| error.to_string())?;
        let key = self
            .store
            .move_in(&staged_path, folder, stem, ext)
            .await
            .map_err(|error| error.to_string())?;
        Ok(meditation_application::ports::StoredBlob {
            storage_key: key.as_str().to_owned(),
            size_bytes: media.size_bytes as i64,
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
/// dir, returning them together with the note/favorite/playback/session ports that
/// user deletion purges directly. A missing ffmpeg logs a warning; uploads
/// surface 503 until the operator installs it.
/// The module dependencies plus the four raw ports that user deletion
/// purges directly (notes, favorites, playback states, sessions).
pub struct MeditationModule {
    pub shared: MeditationShared,
    pub notes: Arc<dyn NoteRepository>,
    pub favorites: Arc<dyn FavoriteRepository>,
    pub playback: Arc<dyn PlaybackStateRepository>,
    pub sessions: Arc<dyn SessionRepository>,
}

pub fn meditation_dependencies(
    connection: DatabaseConnection,
    data_dir: &std::path::Path,
    medioss_origin: Arc<String>,
    upload_limits: UploadLimits,
) -> MeditationModule {
    if !ffmpeg_available() {
        tracing::warn!(
            "ffmpeg not found on PATH: meditation uploads will fail with 503 until it is installed"
        );
    }
    let media_root = data_dir.join("media").join("meditation");
    let staging_root = data_dir.join(crate::bootstrap::scratch_dir::SCRATCH_DIR_NAME);
    let notes: Arc<dyn NoteRepository> = Arc::new(SeaOrmNoteRepository::new(connection.clone()));
    let favorites: Arc<dyn FavoriteRepository> =
        Arc::new(SeaOrmFavoriteRepository::new(connection.clone()));
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
        Arc::clone(&favorites),
        Arc::clone(&playback),
        Arc::clone(&sessions),
        Arc::new(FfmpegAudioValidator),
        Arc::new(CoverNormalizer {
            maximum_output_bytes: upload_limits.program_cover_bytes,
        }),
        Arc::new(FfmpegOpusConverter::new(staging_root.clone())),
        Arc::new(MediaStoreBlobs::new(
            media_store::MediaStore::new(media_root),
            staging_root.clone(),
        )),
        Arc::new(TempFileStaging::new(staging_root)),
        Arc::new(MeditationClock),
        medioss_origin,
        upload_limits.meditation_audio_bytes,
        upload_limits.program_cover_bytes,
    );
    MeditationModule {
        shared: dependencies,
        notes,
        favorites,
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
    favorites: Arc<dyn FavoriteRepository>,
    playback: Arc<dyn PlaybackStateRepository>,
    sessions: Arc<dyn SessionRepository>,
    validator: Arc<dyn meditation_application::ports::AudioValidator>,
    cover_validator: Arc<dyn meditation_application::ports::CoverValidator>,
    converter: Arc<dyn meditation_application::ports::OpusConverter>,
    store: Arc<dyn meditation_application::ports::BlobStore>,
    staging: Arc<dyn meditation_application::ports::UploadStaging>,
    clock: Arc<dyn meditation_application::ports::Clock>,
    medioss_origin: Arc<String>,
    max_upload_bytes: usize,
    max_cover_bytes: usize,
) -> MeditationShared {
    Arc::new(MeditationDependencies {
        create_category: Arc::new(CreateCategoryCommandHandler::new(
            categories.clone(),
            clock.clone(),
        )),
        rename_category: Arc::new(RenameCategoryCommandHandler::new(
            categories.clone(),
            clock.clone(),
        )),
        delete_category: Arc::new(DeleteCategoryCommandHandler::new(
            categories.clone(),
            programs.clone(),
        )),
        set_program_category: Arc::new(SetProgramCategoryCommandHandler::new(
            programs.clone(),
            categories.clone(),
        )),
        list_categories: Arc::new(ListCategoriesQueryHandler::new(categories.clone())),
        create_program: Arc::new(CreateProgramCommandHandler::new(
            programs.clone(),
            cover_validator.clone(),
            store.clone(),
            clock.clone(),
            max_cover_bytes,
        )),
        update_program: Arc::new(UpdateProgramCommandHandler::new(programs.clone())),
        delete_program: Arc::new(DeleteProgramCommandHandler::new(
            programs.clone(),
            memberships.clone(),
            meditations.clone(),
            store.clone(),
        )),
        list_programs: Arc::new(ListProgramsQueryHandler::new(programs.clone())),
        list_program_members: Arc::new(ListProgramMembersQueryHandler::new(
            programs.clone(),
            memberships.clone(),
        )),
        add_member: Arc::new(AddMemberCommandHandler::new(
            meditations.clone(),
            programs.clone(),
            memberships.clone(),
            clock.clone(),
        )),
        remove_member: Arc::new(RemoveMemberCommandHandler::new(
            programs.clone(),
            memberships.clone(),
        )),
        update_program_order: Arc::new(UpdateProgramOrderCommandHandler::new(
            programs.clone(),
            memberships.clone(),
        )),
        upload_meditation: Arc::new(UploadMeditationCommandHandler::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            validator,
            converter,
            store.clone(),
            staging.clone(),
            clock.clone(),
            max_upload_bytes,
        )),
        delete_meditation: Arc::new(DeleteMeditationCommandHandler::new(
            meditations.clone(),
            memberships.clone(),
            notes.clone(),
            favorites.clone(),
            store.clone(),
        )),
        update_meditation_description: Arc::new(UpdateMeditationDescriptionCommandHandler::new(
            meditations.clone(),
        )),
        get_meditation: Arc::new(GetMeditationQueryHandler::new(meditations.clone())),
        get_meditation_detail: Arc::new(GetMeditationDetailQueryHandler::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        get_daily_suggestion: Arc::new(GetDailySuggestionQueryHandler::new(
            meditations.clone(),
            clock.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_library: Arc::new(ListLibraryQueryHandler::new(
            meditations.clone(),
            memberships.clone(),
            programs.clone(),
            categories,
        )),
        get_program_cover: Arc::new(GetProgramCoverQueryHandler::new(
            programs.clone(),
            store.clone(),
        )),
        replace_program_cover: Arc::new(ReplaceProgramCoverCommandHandler::new(
            programs.clone(),
            store.clone(),
        )),
        get_playback_state: Arc::new(GetPlaybackStateQueryHandler::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_position: Arc::new(RecordPositionCommandHandler::new(
            meditations.clone(),
            playback.clone(),
        )),
        record_completion: Arc::new(RecordCompletionCommandHandler::new(
            meditations.clone(),
            playback.clone(),
            sessions.clone(),
        )),
        set_completed: Arc::new(SetCompletedCommandHandler::new(
            meditations.clone(),
            playback.clone(),
        )),
        get_user_statistics: Arc::new(GetUserStatisticsQueryHandler::new(
            sessions.clone(),
            clock.clone(),
        )),
        create_note: Arc::new(CreateNoteCommandHandler::new(
            meditations.clone(),
            notes.clone(),
            clock.clone(),
        )),
        update_note: Arc::new(UpdateNoteCommandHandler::new(notes.clone(), clock.clone())),
        delete_note: Arc::new(DeleteNoteCommandHandler::new(notes.clone())),
        favorite_meditation: Arc::new(FavoriteMeditationCommandHandler::new(
            meditations.clone(),
            favorites.clone(),
            clock.clone(),
        )),
        unfavorite_meditation: Arc::new(UnfavoriteMeditationCommandHandler::new(
            meditations.clone(),
            favorites.clone(),
        )),
        list_favorite_meditations: Arc::new(ListFavoriteMeditationsQueryHandler::new(
            meditations.clone(),
            favorites.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_favorite_ids: Arc::new(ListFavoriteIdsQueryHandler::new(favorites)),
        list_notes: Arc::new(ListNotesQueryHandler::new(meditations, notes)),
        get_continue_program: Arc::new(GetContinueProgramQueryHandler::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        list_recent_programs: Arc::new(ListRecentProgramsQueryHandler::new(
            playback.clone(),
            memberships.clone(),
            programs.clone(),
        )),
        programs,
        playback,
        cover_validator,
        store,
        staging,
        medioss_origin,
        max_upload_bytes,
        max_cover_bytes,
    })
}

const MAX_COVER_DIMENSION: u32 = 1_024;
struct CoverNormalizer {
    maximum_output_bytes: usize,
}

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
                maximum_output_bytes: self.maximum_output_bytes,
                square: false,
            },
        )
        .await
        .map_err(meditation_application::ports::CoverNormalizationError::from)
    }
}

#[cfg(test)]
#[path = "meditation.tests.rs"]
mod tests;
