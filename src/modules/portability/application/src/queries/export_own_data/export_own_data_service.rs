use std::sync::Arc;

use super::export_own_data::ExportOwnData;
use super::export_own_data_handler::ExportOwnDataHandler;
use crate::models::export_document::ExportDocument;
use crate::ports::display_name_resolver::DisplayNameResolver;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;

pub struct ExportOwnDataService {
    mood: Arc<dyn OwnMoodRecords>,
    practice: Arc<dyn OwnPracticeRecords>,
    profile: Arc<dyn DisplayNameResolver>,
}

impl ExportOwnDataService {
    pub fn new(
        mood: Arc<dyn OwnMoodRecords>,
        practice: Arc<dyn OwnPracticeRecords>,
        profile: Arc<dyn DisplayNameResolver>,
    ) -> Self {
        Self {
            mood,
            practice,
            profile,
        }
    }
}

#[async_trait::async_trait]
impl ExportOwnDataHandler for ExportOwnDataService {
    async fn handle(&self, command: ExportOwnData) -> Result<ExportDocument, String> {
        Ok(ExportDocument {
            format_version: crate::FORMAT_VERSION,
            exported_at: chrono::Utc::now(),
            display_name: self.profile.display_name(command.user_id).await?,
            mood_entries: self.mood.list_for_user(command.user_id).await?,
            practice: self.practice.list_for_user(command.user_id).await?,
        })
    }
}

#[cfg(test)]
#[path = "export_own_data_service.tests.rs"]
mod tests;
