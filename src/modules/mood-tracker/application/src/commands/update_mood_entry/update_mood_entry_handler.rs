use super::update_mood_entry::UpdateMoodEntry;
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

#[async_trait::async_trait]
pub trait UpdateMoodEntryHandler: Send + Sync {
    async fn handle(&self, command: UpdateMoodEntry) -> Result<MoodEntry, ApplicationError>;
}
