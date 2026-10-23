use super::list_sounds::ListSounds;
use crate::ApplicationError;
use soundscape_domain::Sound;

#[async_trait::async_trait]
pub trait ListSoundsHandler: Send + Sync {
    async fn handle(&self, query: ListSounds) -> Result<Vec<Sound>, ApplicationError>;
}
