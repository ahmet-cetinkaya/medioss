use std::sync::Arc;

use super::export_own_data_query::ExportOwnDataQuery;
use super::export_own_data_query_validator::ExportOwnDataQueryValidator;
use crate::models::export_document::ExportDocument;
use crate::ports::display_name_resolver::DisplayNameResolver;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::ports::query_handler::QueryHandler;

pub struct ExportOwnDataQueryHandler {
    mood: Arc<dyn OwnMoodRecords>,
    practice: Arc<dyn OwnPracticeRecords>,
    profile: Arc<dyn DisplayNameResolver>,
    validator: ExportOwnDataQueryValidator,
}

impl ExportOwnDataQueryHandler {
    pub fn new(
        mood: Arc<dyn OwnMoodRecords>,
        practice: Arc<dyn OwnPracticeRecords>,
        profile: Arc<dyn DisplayNameResolver>,
    ) -> Self {
        Self {
            mood,
            practice,
            profile,
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl QueryHandler<ExportOwnDataQuery> for ExportOwnDataQueryHandler {
    type Output = ExportDocument;
    type Error = String;

    async fn handle(&self, query: ExportOwnDataQuery) -> Result<ExportDocument, String> {
        self.validator.validate(&query)?;
        Ok(ExportDocument {
            format_version: crate::FORMAT_VERSION,
            exported_at: chrono::Utc::now(),
            display_name: self.profile.display_name(query.user_id).await?,
            mood_entries: self.mood.list_for_user(query.user_id).await?,
            practice: self.practice.list_for_user(query.user_id).await?,
        })
    }
}

#[cfg(test)]
#[path = "export_own_data_query_handler.tests.rs"]
mod tests;
