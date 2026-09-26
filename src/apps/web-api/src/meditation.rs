//! Meditation wiring in the composition root: repositories over the shared
//! SQLite connection, the media-store-backed blob store, and the ffmpeg
//! adapters.

use std::sync::Arc;

use meditation_application::ports::{
    AudioValidator as _, BlobStore as _, Clock as _, OpusConverter as _,
};
use meditation_infrastructure::persistence::{
    SeaOrmCategoryRepository, SeaOrmMeditationRepository, SeaOrmMembershipRepository,
    SeaOrmNoteRepository, SeaOrmPlaybackRepository, SeaOrmProgramRepository,
    SeaOrmSessionRepository, SystemClock as MeditationClock,
};
use meditation_infrastructure::{media::FfmpegAudioValidator, media::FfmpegOpusConverter};
use meditation_presentation::SharedDependencies as MeditationShared;
use sea_orm::DatabaseConnection;

/// Implements the application BlobStore port over the shared media-store
/// crate. The namespace is fixed per store instance; audio and covers each
/// get their own adapter.
pub struct MediaStoreBlobs {
    store: media_store::MediaStore,
    namespace: media_store::StorageNamespace,
}

impl MediaStoreBlobs {
    /// Builds an adapter over `root` for one namespace.
    pub fn new(store: media_store::MediaStore, namespace: media_store::StorageNamespace) -> Self {
        Self { store, namespace }
    }
}

#[async_trait::async_trait]
impl meditation_application::ports::BlobStore for MediaStoreBlobs {
    async fn put(&self, bytes: &[u8]) -> Result<meditation_application::ports::StoredBlob, String> {
        let key = self
            .store
            .put(self.namespace, bytes)
            .await
            .map_err(|error| error.to_string())?;
        Ok(meditation_application::ports::StoredBlob {
            storage_key: key.as_str().to_owned(),
            size_bytes: bytes.len() as i64,
        })
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        let key = media_store::StorageKey::parse(key).map_err(|error| error.to_string())?;
        self.store
            .delete(self.namespace, &key)
            .await
            .map_err(|error| error.to_string())
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>, String> {
        let key = media_store::StorageKey::parse(key).map_err(|error| error.to_string())?;
        let stream = self
            .store
            .open(self.namespace, &key)
            .await
            .map_err(|error| error.to_string())?;
        let mut reader = stream.reader();
        let mut buffer = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut buffer)
            .await
            .map_err(|error| error.to_string())?;
        Ok(buffer)
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
/// dir. Fails fast is not desired for ffmpeg: a warning is logged and
/// uploads will surface 503 until the operator installs it.
pub fn meditation_dependencies(
    connection: DatabaseConnection,
    data_dir: &std::path::Path,
    medioss_origin: Arc<String>,
) -> MeditationShared {
    if !ffmpeg_available() {
        tracing::warn!(
            "ffmpeg not found on PATH: meditation uploads will fail with 503 until it is installed"
        );
    }
    let media_root = data_dir.join("media");
    let audio_blobs = MediaStoreBlobs::new(
        media_store::MediaStore::new(media_root),
        media_store::StorageNamespace::Meditation,
    );
    Arc::new(meditation_presentation::MeditationDependencies {
        meditations: Arc::new(SeaOrmMeditationRepository::new(connection.clone())),
        programs: Arc::new(SeaOrmProgramRepository::new(connection.clone())),
        categories: Arc::new(SeaOrmCategoryRepository::new(connection.clone())),
        memberships: Arc::new(SeaOrmMembershipRepository::new(connection.clone())),
        notes: Arc::new(SeaOrmNoteRepository::new(connection.clone())),
        playback: Arc::new(SeaOrmPlaybackRepository::new(connection.clone())),
        sessions: Arc::new(SeaOrmSessionRepository::new(connection)),
        validator: Arc::new(FfmpegAudioValidator),
        cover_validator: Arc::new(CoverSniffer),
        converter: Arc::new(FfmpegOpusConverter),
        store: Arc::new(audio_blobs),
        clock: Arc::new(MeditationClock),
        medioss_origin,
    })
}

/// Cover sniffing without a subprocess: pure magic-byte rules.
struct CoverSniffer;

#[async_trait::async_trait]
impl meditation_application::ports::CoverValidator for CoverSniffer {
    async fn sniff(
        &self,
        leading: &[u8],
    ) -> Result<meditation_application::ports::ImageFormat, meditation_application::ports::SniffError>
    {
        if leading.len() < 12 {
            return Err(meditation_application::ports::SniffError::TooShort);
        }
        if leading.starts_with(&[0x89, b'P', b'N', b'G']) {
            return Ok(meditation_application::ports::ImageFormat::Png);
        }
        if leading.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(meditation_application::ports::ImageFormat::Jpeg);
        }
        if leading.starts_with(b"RIFF") && &leading[8..12] == b"WEBP" {
            return Ok(meditation_application::ports::ImageFormat::Webp);
        }
        Err(meditation_application::ports::SniffError::Unsupported)
    }
}
