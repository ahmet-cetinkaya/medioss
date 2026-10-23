use super::get_user_statistics::GetUserStatistics;
use super::user_statistics::UserStatistics;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetUserStatisticsHandler: Send + Sync {
    async fn handle(&self, query: GetUserStatistics) -> Result<UserStatistics, ApplicationError>;
}
