use super::setup_status::IsSetupRequired;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait IsSetupRequiredHandler: Send + Sync {
    async fn handle(&self, query: IsSetupRequired) -> Result<bool, ApplicationError>;
}
