use sea_orm::DatabaseConnection;
use soundscape_infrastructure::media::{
    FfmpegAudioValidator, FfmpegOpusConverter, MediaStoreBlobs,
};
use soundscape_presentation::{SharedDependencies, SoundscapeDependencies};
use std::sync::Arc;

pub struct SystemClock;
#[async_trait::async_trait]
impl soundscape_application::ports::Clock for SystemClock {
    async fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

pub fn soundscape_dependencies(
    db: DatabaseConnection,
    data_dir: &std::path::Path,
    medioss_origin: Arc<String>,
) -> SharedDependencies {
    Arc::new(SoundscapeDependencies {
        categories: Arc::new(
            soundscape_infrastructure::persistence::SeaOrmCategoryRepository::new(db.clone()),
        ),
        sounds: Arc::new(
            soundscape_infrastructure::persistence::SeaOrmSoundRepository::new(db.clone()),
        ),
        mixes: Arc::new(soundscape_infrastructure::persistence::SeaOrmMixRepository::new(db)),
        clock: Arc::new(SystemClock),
        validator: Arc::new(FfmpegAudioValidator),
        converter: Arc::new(FfmpegOpusConverter),
        store: Arc::new(MediaStoreBlobs(media_store::MediaStore::new(
            data_dir.join("media"),
        ))),
        medioss_origin,
    })
}
