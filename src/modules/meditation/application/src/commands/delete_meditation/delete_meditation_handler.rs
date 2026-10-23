use super::delete_meditation::DeleteMeditation;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteMeditationHandler: Send + Sync {
    async fn handle(&self, command: DeleteMeditation) -> Result<(), ApplicationError>;
}
