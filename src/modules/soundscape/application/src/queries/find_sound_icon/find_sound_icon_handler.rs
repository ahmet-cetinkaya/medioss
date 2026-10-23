use super::find_sound_icon::FindSoundIcon;
use crate::ApplicationError;
use soundscape_domain::SoundIcon;

#[async_trait::async_trait]
pub trait FindSoundIconHandler: Send + Sync {
    async fn handle(&self, query: FindSoundIcon) -> Result<SoundIcon, ApplicationError>;
}
