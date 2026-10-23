use super::create_mix::CreateMix;
use crate::ApplicationError;
use soundscape_domain::Mix;

#[async_trait::async_trait]
pub trait CreateMixHandler: Send + Sync {
    async fn handle(&self, command: CreateMix) -> Result<Mix, ApplicationError>;
}
