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
    use soundscape_application::commands::clear_sound_icon::ClearSoundIconService;
    use soundscape_application::commands::create_category::CreateCategoryService;
    use soundscape_application::commands::create_mix::CreateMixService;
    use soundscape_application::commands::create_sound::CreateSoundService;
    use soundscape_application::commands::delete_category::DeleteCategoryService;
    use soundscape_application::commands::delete_mix::DeleteMixService;
    use soundscape_application::commands::delete_sound::DeleteSoundService;
    use soundscape_application::commands::rename_category::RenameCategoryService;
    use soundscape_application::commands::set_sound_icon::SetSoundIconService;
    use soundscape_application::commands::update_mix::UpdateMixService;
    use soundscape_application::commands::update_sound::UpdateSoundService;
    use soundscape_application::queries::find_sound_icon::FindSoundIconService;
    use soundscape_application::queries::list_categories::ListCategoriesService;
    use soundscape_application::queries::list_sounds::ListSoundsService;
    use soundscape_infrastructure::persistence::{
        SeaOrmCategoryRepository, SeaOrmMixRepository, SeaOrmSoundRepository,
    };

    let categories: Arc<dyn soundscape_application::ports::CategoryRepository> =
        Arc::new(SeaOrmCategoryRepository::new(db.clone()));
    let sounds: Arc<dyn soundscape_application::ports::SoundRepository> =
        Arc::new(SeaOrmSoundRepository::new(db.clone()));
    let mixes: Arc<dyn soundscape_application::ports::MixRepository> =
        Arc::new(SeaOrmMixRepository::new(db.clone()));
    let store: Arc<dyn soundscape_application::ports::media::BlobStore> = Arc::new(
        MediaStoreBlobs(media_store::MediaStore::new(data_dir.join("media"))),
    );
    let normalizer: Arc<dyn soundscape_application::ports::media::ImageNormalizer> =
        Arc::new(IconNormalizer);

    Arc::new(SoundscapeDependencies {
        list_categories: Arc::new(ListCategoriesService::new(categories.clone())),
        list_sounds: Arc::new(ListSoundsService::new(sounds.clone())),
        find_sound_icon: Arc::new(FindSoundIconService::new(sounds.clone())),
        create_category: Arc::new(CreateCategoryService::new(categories.clone())),
        rename_category: Arc::new(RenameCategoryService::new(categories.clone())),
        delete_category: Arc::new(DeleteCategoryService::new(categories.clone())),
        create_sound: Arc::new(CreateSoundService::new(
            categories.clone(),
            sounds.clone(),
            Arc::new(FfmpegAudioValidator),
            Arc::new(FfmpegOpusConverter),
            store.clone(),
            normalizer.clone(),
        )),
        update_sound: Arc::new(UpdateSoundService::new(categories.clone(), sounds.clone())),
        delete_sound: Arc::new(DeleteSoundService::new(sounds.clone(), store.clone())),
        set_sound_icon: Arc::new(SetSoundIconService::new(
            sounds.clone(),
            store.clone(),
            normalizer.clone(),
        )),
        clear_sound_icon: Arc::new(ClearSoundIconService::new(sounds.clone(), store.clone())),
        create_mix: Arc::new(CreateMixService::new(
            mixes.clone(),
            sounds.clone(),
            Arc::new(SystemClock),
        )),
        update_mix: Arc::new(UpdateMixService::new(
            mixes.clone(),
            sounds.clone(),
            Arc::new(SystemClock),
        )),
        delete_mix: Arc::new(DeleteMixService::new(mixes.clone())),
        store,
        sounds,
        mixes,
        medioss_origin,
    })
}

#[cfg(test)]
#[path = "soundscape.tests.rs"]
mod tests;
