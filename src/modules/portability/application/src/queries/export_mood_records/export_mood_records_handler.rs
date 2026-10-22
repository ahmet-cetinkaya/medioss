use super::export_mood_records::ExportMoodRecords;
use crate::models::mood_record::MoodRecord;

#[async_trait::async_trait]
pub trait ExportMoodRecordsHandler: Send + Sync {
    async fn handle(&self, command: ExportMoodRecords) -> Result<Vec<MoodRecord>, String>;
}
