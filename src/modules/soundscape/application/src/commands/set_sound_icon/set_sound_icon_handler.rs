use super::set_sound_icon::SetSoundIcon;
use crate::ApplicationError;
use soundscape_domain::Sound;

#[async_trait::async_trait]
pub trait SetSoundIconHandler: Send + Sync {
    async fn handle(&self, command: SetSoundIcon) -> Result<Sound, ApplicationError>;
}
