use std::sync::Arc;

use super::import_own_data_handler::ImportOwnDataHandler;
use super::mood::{apply_mood_entries, existing_mood_keys};
use super::practice::apply_practice;
use super::{import_own_data::ImportFailure, import_own_data::ImportOwnData};
use crate::models::import_report::ImportReport;
use crate::ports::meditation_lookup::MeditationLookup;
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::ports::practice_sink::PracticeSink;
use crate::queries::validate_import::validate_document;

pub struct ImportOwnDataService {
    mood_source: Arc<dyn OwnMoodRecords>,
    mood_sink: Arc<dyn MoodRecordSink>,
    practice_source: Arc<dyn OwnPracticeRecords>,
    practice_sink: Arc<dyn PracticeSink>,
    meditations: Arc<dyn MeditationLookup>,
}

impl ImportOwnDataService {
    pub fn new(
        mood_source: Arc<dyn OwnMoodRecords>,
        mood_sink: Arc<dyn MoodRecordSink>,
        practice_source: Arc<dyn OwnPracticeRecords>,
        practice_sink: Arc<dyn PracticeSink>,
        meditations: Arc<dyn MeditationLookup>,
    ) -> Self {
        Self {
            mood_source,
            mood_sink,
            practice_source,
            practice_sink,
            meditations,
        }
    }
}

#[async_trait::async_trait]
impl ImportOwnDataHandler for ImportOwnDataService {
    async fn handle(&self, command: ImportOwnData) -> Result<ImportReport, ImportFailure> {
        validate_document(&command.document)
            .map_err(|e| ImportFailure::Validation(e.to_string()))?;

        let existing_moods = existing_mood_keys(command.user_id, self.mood_source.as_ref()).await?;
        let all_meditations = self.meditations.list_all().await?;
        let index = super::meditation_index::meditation_index(&all_meditations);
        let mut report = ImportReport::default();

        apply_mood_entries(
            command.user_id,
            command.document.mood_entries,
            existing_moods,
            self.mood_sink.as_ref(),
            &mut report,
        )
        .await?;

        let existing_practice = self.practice_source.list_for_user(command.user_id).await?;
        apply_practice(
            command.user_id,
            command.document.practice,
            &existing_practice,
            &index,
            self.practice_sink.as_ref(),
            &mut report,
        )
        .await?;

        Ok(report)
    }
}

#[cfg(test)]
#[path = "import_own_data_service.tests.rs"]
mod tests;
