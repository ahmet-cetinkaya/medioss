use super::delete_sound::DeleteSound;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteSoundHandler: Send + Sync {
    async fn handle(&self, command: DeleteSound) -> Result<(), ApplicationError>;
}
