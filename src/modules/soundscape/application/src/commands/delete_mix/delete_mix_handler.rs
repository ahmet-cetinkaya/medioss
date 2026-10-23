use super::delete_mix::DeleteMix;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteMixHandler: Send + Sync {
    async fn handle(&self, command: DeleteMix) -> Result<(), ApplicationError>;
}
