use super::update_meditation_description::UpdateMeditationDescription;
use crate::ApplicationError;
use meditation_domain::Meditation;

#[async_trait::async_trait]
pub trait UpdateMeditationDescriptionHandler: Send + Sync {
    async fn handle(
        &self,
        command: UpdateMeditationDescription,
    ) -> Result<Meditation, ApplicationError>;
}
