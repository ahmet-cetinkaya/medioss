use super::record_completion::RecordCompletion;
use crate::ports::MeditationSession;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait RecordCompletionHandler: Send + Sync {
    async fn handle(
        &self,
        command: RecordCompletion,
    ) -> Result<MeditationSession, ApplicationError>;
}
