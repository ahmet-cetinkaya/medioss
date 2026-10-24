use std::sync::Arc;

use super::import_own_data_command_validator::ImportOwnDataCommandValidator;
use super::mood::{apply_mood_entries, existing_mood_keys};
use super::practice::apply_practice;
use super::{
    import_own_data_command::ImportFailure, import_own_data_command::ImportOwnDataCommand,
};
use crate::models::import_report::ImportReport;
use crate::ports::command_handler::CommandHandler;
use crate::ports::meditation_lookup::MeditationLookup;
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::ports::practice_sink::PracticeSink;

pub struct ImportOwnDataCommandHandler {
    mood_source: Arc<dyn OwnMoodRecords>,
    mood_sink: Arc<dyn MoodRecordSink>,
    practice_source: Arc<dyn OwnPracticeRecords>,
    practice_sink: Arc<dyn PracticeSink>,
    meditations: Arc<dyn MeditationLookup>,
    validator: ImportOwnDataCommandValidator,
}

impl ImportOwnDataCommandHandler {
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
            validator: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler<ImportOwnDataCommand> for ImportOwnDataCommandHandler {
    type Output = ImportReport;
    type Error = ImportFailure;

    async fn handle(&self, command: ImportOwnDataCommand) -> Result<ImportReport, ImportFailure> {
        self.validator.validate(&command)?;

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
#[path = "import_own_data_command_handler.tests.rs"]
mod tests;
