use super::create_mood_entry::CreateMoodEntry;
use crate::ApplicationError;
use mood_tracker_domain::MoodEntry;

#[async_trait::async_trait]
pub trait CreateMoodEntryHandler: Send + Sync {
    async fn handle(&self, command: CreateMoodEntry) -> Result<MoodEntry, ApplicationError>;
}
