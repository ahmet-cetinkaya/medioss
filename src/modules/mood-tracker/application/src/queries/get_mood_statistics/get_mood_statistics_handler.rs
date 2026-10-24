use super::get_mood_statistics::GetMoodStatistics;
use super::mood_statistics::MoodStatistics;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetMoodStatisticsHandler: Send + Sync {
    async fn handle(&self, query: GetMoodStatistics) -> Result<MoodStatistics, ApplicationError>;
}
