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
    max_upload_bytes: usize,
) -> SharedDependencies {
    use soundscape_application::commands::clear_sound_icon::ClearSoundIconCommandHandler;
    use soundscape_application::commands::create_category::CreateCategoryCommandHandler;
    use soundscape_application::commands::create_mix::CreateMixCommandHandler;
    use soundscape_application::commands::create_sound::CreateSoundCommandHandler;
    use soundscape_application::commands::delete_category::DeleteCategoryCommandHandler;
    use soundscape_application::commands::delete_mix::DeleteMixCommandHandler;
    use soundscape_application::commands::delete_sound::DeleteSoundCommandHandler;
    use soundscape_application::commands::rename_category::RenameCategoryCommandHandler;
    use soundscape_application::commands::set_sound_icon::SetSoundIconCommandHandler;
    use soundscape_application::commands::update_mix::UpdateMixCommandHandler;
    use soundscape_application::commands::update_sound::UpdateSoundCommandHandler;
    use soundscape_application::queries::find_sound_icon::FindSoundIconQueryHandler;
    use soundscape_application::queries::list_categories::ListCategoriesQueryHandler;
    use soundscape_application::queries::list_sounds::ListSoundsQueryHandler;
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
        list_categories: Arc::new(ListCategoriesQueryHandler::new(categories.clone())),
        list_sounds: Arc::new(ListSoundsQueryHandler::new(sounds.clone())),
        find_sound_icon: Arc::new(FindSoundIconQueryHandler::new(sounds.clone())),
        create_category: Arc::new(CreateCategoryCommandHandler::new(categories.clone())),
        rename_category: Arc::new(RenameCategoryCommandHandler::new(categories.clone())),
        delete_category: Arc::new(DeleteCategoryCommandHandler::new(categories.clone())),
        create_sound: Arc::new(CreateSoundCommandHandler::new(
            categories.clone(),
            sounds.clone(),
            Arc::new(FfmpegAudioValidator),
            Arc::new(FfmpegOpusConverter),
            store.clone(),
            normalizer.clone(),
            max_upload_bytes,
        )),
        update_sound: Arc::new(UpdateSoundCommandHandler::new(
            categories.clone(),
            sounds.clone(),
        )),
        delete_sound: Arc::new(DeleteSoundCommandHandler::new(
            sounds.clone(),
            store.clone(),
        )),
        set_sound_icon: Arc::new(SetSoundIconCommandHandler::new(
            sounds.clone(),
            store.clone(),
            normalizer.clone(),
        )),
        clear_sound_icon: Arc::new(ClearSoundIconCommandHandler::new(
            sounds.clone(),
            store.clone(),
        )),
        create_mix: Arc::new(CreateMixCommandHandler::new(
            mixes.clone(),
            sounds.clone(),
            Arc::new(SystemClock),
        )),
        update_mix: Arc::new(UpdateMixCommandHandler::new(
            mixes.clone(),
            sounds.clone(),
            Arc::new(SystemClock),
        )),
        delete_mix: Arc::new(DeleteMixCommandHandler::new(mixes.clone())),
        store,
        sounds,
        mixes,
        medioss_origin,
        max_upload_bytes,
    })
}

#[cfg(test)]
#[path = "soundscape.tests.rs"]
mod tests;
