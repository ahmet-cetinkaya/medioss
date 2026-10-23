use super::create_sound::CreateSound;
use crate::ApplicationError;
use soundscape_domain::Sound;

#[async_trait::async_trait]
pub trait CreateSoundHandler: Send + Sync {
    async fn handle(&self, command: CreateSound) -> Result<Sound, ApplicationError>;
}
