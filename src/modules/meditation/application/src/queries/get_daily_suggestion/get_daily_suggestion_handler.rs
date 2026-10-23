use super::daily_suggestion::DailySuggestion;
use super::get_daily_suggestion::GetDailySuggestion;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait GetDailySuggestionHandler: Send + Sync {
    async fn handle(
        &self,
        query: GetDailySuggestion,
    ) -> Result<Option<DailySuggestion>, ApplicationError>;
}
