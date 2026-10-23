use super::update_mix::UpdateMix;
use crate::ApplicationError;
use soundscape_domain::Mix;

#[async_trait::async_trait]
pub trait UpdateMixHandler: Send + Sync {
    async fn handle(&self, command: UpdateMix) -> Result<Mix, ApplicationError>;
}
