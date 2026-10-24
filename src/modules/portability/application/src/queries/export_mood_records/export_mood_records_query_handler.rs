use std::sync::Arc;

use super::export_mood_records_query::ExportMoodRecordsQuery;
use super::export_mood_records_query_validator::ExportMoodRecordsQueryValidator;
use crate::models::mood_record::MoodRecord;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::query_handler::QueryHandler;

pub struct ExportMoodRecordsQueryHandler {
    mood: Arc<dyn OwnMoodRecords>,
    validator: ExportMoodRecordsQueryValidator,
}

impl ExportMoodRecordsQueryHandler {
    pub fn new(mood: Arc<dyn OwnMoodRecords>) -> Self {
        Self {
            mood,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ExportMoodRecordsQuery> for ExportMoodRecordsQueryHandler {
    type Output = Vec<MoodRecord>;
    type Error = String;

    async fn handle(&self, query: ExportMoodRecordsQuery) -> Result<Vec<MoodRecord>, String> {
        self.validator.validate(&query)?;
        self.mood.list_for_user(query.user_id).await
    }
}

#[cfg(test)]
#[path = "export_mood_records_query_handler.tests.rs"]
mod tests;
