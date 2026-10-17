use sea_orm::DatabaseConnection;
use soundscape_domain::MAX_ICON_BYTES;
use soundscape_infrastructure::media::{
    FfmpegAudioValidator, FfmpegOpusConverter, MediaStoreBlobs,
};
use soundscape_presentation::{SharedDependencies, SoundscapeDependencies};
use std::sync::Arc;

const MAX_ICON_DIMENSION: u32 = 512;
const MAX_ICON_OUTPUT_BYTES: usize = MAX_ICON_BYTES;

pub struct IconNormalizer;

#[async_trait::async_trait]
impl soundscape_application::ports::media::ImageNormalizer for IconNormalizer {
    async fn normalize(
        &self,
        bytes: Vec<u8>,
    ) -> Result<Vec<u8>, soundscape_application::ports::media::ImageNormalizationError> {
        crate::image_normalization::normalize(
            bytes,
            crate::image_normalization::ImageSpec {
                maximum_dimension: MAX_ICON_DIMENSION,
                maximum_output_bytes: MAX_ICON_OUTPUT_BYTES,
                square: false,
            },
        )
        .await
        .map_err(soundscape_application::ports::media::ImageNormalizationError::from)
    }
}

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
        image_normalizer: Arc::new(IconNormalizer),
        medioss_origin,
    })
}

#[cfg(test)]
#[path = "soundscape.tests.rs"]
mod tests;
