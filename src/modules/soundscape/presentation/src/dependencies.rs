use std::sync::Arc;

use soundscape_application::ports::{
    media::{AudioValidator, BlobStore, ImageNormalizer, OpusConverter},
    CategoryRepository, Clock, MixRepository, SoundRepository,
};

pub struct SoundscapeDependencies {
    pub categories: Arc<dyn CategoryRepository>,
    pub sounds: Arc<dyn SoundRepository>,
    pub mixes: Arc<dyn MixRepository>,
    pub clock: Arc<dyn Clock>,
    pub validator: Arc<dyn AudioValidator>,
    pub converter: Arc<dyn OpusConverter>,
    pub store: Arc<dyn BlobStore>,
    pub image_normalizer: Arc<dyn ImageNormalizer>,
    pub medioss_origin: Arc<String>,
}
pub type SharedDependencies = Arc<SoundscapeDependencies>;
