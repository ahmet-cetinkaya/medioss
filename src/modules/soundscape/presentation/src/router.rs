use std::sync::Arc;

use axum::Router;
use soundscape_application::commands::clear_sound_icon::ClearSoundIconHandler;
use soundscape_application::commands::create_category::CreateCategoryHandler;
use soundscape_application::commands::create_mix::CreateMixHandler;
use soundscape_application::commands::create_sound::CreateSoundHandler;
use soundscape_application::commands::delete_category::DeleteCategoryHandler;
use soundscape_application::commands::delete_mix::DeleteMixHandler;
use soundscape_application::commands::delete_sound::DeleteSoundHandler;
use soundscape_application::commands::rename_category::RenameCategoryHandler;
use soundscape_application::commands::set_sound_icon::SetSoundIconHandler;
use soundscape_application::commands::update_mix::UpdateMixHandler;
use soundscape_application::commands::update_sound::UpdateSoundHandler;
use soundscape_application::ports::media::BlobStore;
use soundscape_application::ports::{MixRepository, SoundRepository};
use soundscape_application::queries::find_sound_icon::FindSoundIconHandler;
use soundscape_application::queries::list_categories::ListCategoriesHandler;
use soundscape_application::queries::list_sounds::ListSoundsHandler;

/// Everything the soundscape routes need, resolved at the composition root.
/// Use-case handlers arrive as traits; the repository and blob-store ports
/// stay raw because icon/audio byte reads and per-user mix reads are pure
/// reads no use case wraps.
#[derive(Clone)]
pub struct SoundscapeDependencies {
    pub list_categories: Arc<dyn ListCategoriesHandler>,
    pub list_sounds: Arc<dyn ListSoundsHandler>,
    pub find_sound_icon: Arc<dyn FindSoundIconHandler>,
    pub create_category: Arc<dyn CreateCategoryHandler>,
    pub rename_category: Arc<dyn RenameCategoryHandler>,
    pub delete_category: Arc<dyn DeleteCategoryHandler>,
    pub create_sound: Arc<dyn CreateSoundHandler>,
    pub update_sound: Arc<dyn UpdateSoundHandler>,
    pub delete_sound: Arc<dyn DeleteSoundHandler>,
    pub set_sound_icon: Arc<dyn SetSoundIconHandler>,
    pub clear_sound_icon: Arc<dyn ClearSoundIconHandler>,
    pub create_mix: Arc<dyn CreateMixHandler>,
    pub update_mix: Arc<dyn UpdateMixHandler>,
    pub delete_mix: Arc<dyn DeleteMixHandler>,
    pub store: Arc<dyn BlobStore>,
    pub sounds: Arc<dyn SoundRepository>,
    pub mixes: Arc<dyn MixRepository>,
    pub medioss_origin: Arc<String>,
}
pub type SharedDependencies = Arc<SoundscapeDependencies>;

pub fn soundscape_router() -> Router<SharedDependencies> {
    let router = Router::new();
    let router = crate::handlers::categories::register(router);
    let router = crate::handlers::sounds::register(router);
    let router = crate::handlers::sounds_id::register(router);
    let router = crate::handlers::sounds_icon::register(router);
    let router = crate::handlers::sounds_audio::register(router);
    crate::handlers::mixes::register(router)
}
