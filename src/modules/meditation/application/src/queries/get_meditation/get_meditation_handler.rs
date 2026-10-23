use super::get_meditation::GetMeditation;
use crate::ApplicationError;
use meditation_domain::Meditation;

#[async_trait::async_trait]
pub trait GetMeditationHandler: Send + Sync {
    async fn handle(&self, query: GetMeditation) -> Result<Meditation, ApplicationError>;
}
