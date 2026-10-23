use super::upload_meditation::UploadMeditation;
use crate::ApplicationError;
use meditation_domain::Meditation;

#[async_trait::async_trait]
pub trait UploadMeditationHandler: Send + Sync {
    async fn handle(&self, command: UploadMeditation) -> Result<Meditation, ApplicationError>;
}
