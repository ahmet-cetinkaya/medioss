use super::update_sound::UpdateSound;
use crate::ApplicationError;
use soundscape_domain::Sound;

#[async_trait::async_trait]
pub trait UpdateSoundHandler: Send + Sync {
    async fn handle(&self, command: UpdateSound) -> Result<Sound, ApplicationError>;
}
