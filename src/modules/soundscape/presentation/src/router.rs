use soundscape_application::commands::clear_sound_icon::ClearSoundIconCommand;
use soundscape_application::commands::create_category::CreateCategoryCommand;
use soundscape_application::commands::create_mix::CreateMixCommand;
use soundscape_application::commands::create_sound::CreateSoundCommand;
use soundscape_application::commands::delete_category::DeleteCategoryCommand;
use soundscape_application::commands::delete_mix::DeleteMixCommand;
use soundscape_application::commands::delete_sound::DeleteSoundCommand;
use soundscape_application::commands::rename_category::RenameCategoryCommand;
use soundscape_application::commands::set_sound_icon::SetSoundIconCommand;
use soundscape_application::commands::update_mix::UpdateMixCommand;
use soundscape_application::commands::update_sound::UpdateSoundCommand;
use soundscape_application::ports::{CommandHandler, QueryHandler};
use soundscape_application::queries::find_sound_icon::FindSoundIconQuery;
use soundscape_application::queries::list_categories::ListCategoriesQuery;
use soundscape_application::queries::list_sounds::ListSoundsQuery;
use soundscape_domain::{Mix, Sound, SoundCategory, SoundIcon};
use std::sync::Arc;

use axum::Router;
use soundscape_application::ports::media::BlobStore;
use soundscape_application::ports::{MixRepository, SoundRepository};

/// Everything the soundscape routes need, resolved at the composition root.
/// Use-case handlers arrive as traits; the repository and blob-store ports
/// stay raw because icon/audio byte reads and per-user mix reads are pure
/// reads no use case wraps.
#[derive(Clone)]
pub struct SoundscapeDependencies {
    pub list_categories: Arc<dyn QueryHandler<ListCategoriesQuery, Output = Vec<SoundCategory>>>,
    pub list_sounds: Arc<dyn QueryHandler<ListSoundsQuery, Output = Vec<Sound>>>,
    pub find_sound_icon: Arc<dyn QueryHandler<FindSoundIconQuery, Output = SoundIcon>>,
    pub create_category: Arc<dyn CommandHandler<CreateCategoryCommand, Output = SoundCategory>>,
    pub rename_category: Arc<dyn CommandHandler<RenameCategoryCommand, Output = SoundCategory>>,
    pub delete_category: Arc<dyn CommandHandler<DeleteCategoryCommand, Output = ()>>,
    pub create_sound: Arc<dyn CommandHandler<CreateSoundCommand, Output = Sound>>,
    pub update_sound: Arc<dyn CommandHandler<UpdateSoundCommand, Output = Sound>>,
    pub delete_sound: Arc<dyn CommandHandler<DeleteSoundCommand, Output = ()>>,
    pub set_sound_icon: Arc<dyn CommandHandler<SetSoundIconCommand, Output = Sound>>,
    pub clear_sound_icon: Arc<dyn CommandHandler<ClearSoundIconCommand, Output = Sound>>,
    pub create_mix: Arc<dyn CommandHandler<CreateMixCommand, Output = Mix>>,
    pub update_mix: Arc<dyn CommandHandler<UpdateMixCommand, Output = Mix>>,
    pub delete_mix: Arc<dyn CommandHandler<DeleteMixCommand, Output = ()>>,
    pub store: Arc<dyn BlobStore>,
    pub sounds: Arc<dyn SoundRepository>,
    pub mixes: Arc<dyn MixRepository>,
    pub medioss_origin: Arc<String>,
    /// Cap on a sound upload's audio field; `soundscape_router` must receive the same value.
    pub max_upload_bytes: usize,
}
pub type SharedDependencies = Arc<SoundscapeDependencies>;

pub fn soundscape_router(max_upload_bytes: usize) -> Router<SharedDependencies> {
    let router = Router::new();
    let router = crate::handlers::categories::register(router);
    let router = crate::handlers::sounds::register(router, max_upload_bytes);
    let router = crate::handlers::sounds_id::register(router);
    let router = crate::handlers::sounds_icon::register(router);
    let router = crate::handlers::sounds_audio::register(router);
    crate::handlers::mixes::register(router)
}

#[cfg(test)]
#[path = "router.tests.rs"]
mod tests;
