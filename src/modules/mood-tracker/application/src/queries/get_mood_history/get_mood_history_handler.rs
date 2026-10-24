use super::get_mood_history::GetMoodHistory;
use crate::ports::Page;
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

#[async_trait::async_trait]
pub trait GetMoodHistoryHandler: Send + Sync {
    async fn handle(&self, query: GetMoodHistory) -> Result<Page<MoodEntry>, ApplicationError>;
}
