use super::delete_mood_entry::DeleteMoodEntry;
use crate::ApplicationError;

#[async_trait::async_trait]
pub trait DeleteMoodEntryHandler: Send + Sync {
    async fn handle(&self, command: DeleteMoodEntry) -> Result<(), ApplicationError>;
}
