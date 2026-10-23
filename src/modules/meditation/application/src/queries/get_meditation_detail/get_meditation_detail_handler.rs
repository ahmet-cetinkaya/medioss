use super::get_meditation_detail::GetMeditationDetail;
use super::meditation_detail::MeditationDetail;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetMeditationDetailHandler: Send + Sync {
    async fn handle(
        &self,
        query: GetMeditationDetail,
    ) -> Result<MeditationDetail, ApplicationError>;
}
