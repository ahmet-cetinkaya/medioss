use super::clear_sound_icon::ClearSoundIcon;
use crate::ApplicationError;
use soundscape_domain::Sound;

#[async_trait::async_trait]
pub trait ClearSoundIconHandler: Send + Sync {
    async fn handle(&self, command: ClearSoundIcon) -> Result<Sound, ApplicationError>;
}
