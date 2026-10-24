use super::get_mood_history::MoodHistory;
use crate::ports::Page;
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

#[async_trait::async_trait]
pub trait MoodHistoryHandler: Send + Sync {
    async fn handle(&self, query: MoodHistory) -> Result<Page<MoodEntry>, ApplicationError>;
}
