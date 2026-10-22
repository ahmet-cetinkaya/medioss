use std::sync::Arc;

use super::export_mood_records::ExportMoodRecords;
use super::export_mood_records_handler::ExportMoodRecordsHandler;
use crate::models::mood_record::MoodRecord;
use crate::ports::own_mood_records::OwnMoodRecords;

pub struct ExportMoodRecordsService {
    mood: Arc<dyn OwnMoodRecords>,
}

impl ExportMoodRecordsService {
    pub fn new(mood: Arc<dyn OwnMoodRecords>) -> Self {
        Self { mood }
    }
}

#[async_trait::async_trait]
impl ExportMoodRecordsHandler for ExportMoodRecordsService {
    async fn handle(&self, command: ExportMoodRecords) -> Result<Vec<MoodRecord>, String> {
        self.mood.list_for_user(command.user_id).await
    }
}

#[cfg(test)]
#[path = "export_mood_records_service.tests.rs"]
mod tests;
