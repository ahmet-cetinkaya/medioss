mod meditation_index;
mod mood;
mod practice;
mod validate;

#[cfg(test)]
#[path = "import.tests.rs"]
mod import_tests;

use crate::models::export_document::ExportDocument;
use crate::models::import_report::ImportReport;
use crate::ports::meditation_lookup::MeditationLookup;
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::ports::practice_sink::PracticeSink;
use kernel::UserId;

#[derive(Debug)]
pub enum ImportFailure {
    Validation(String),
    Internal(String),
}

impl From<String> for ImportFailure {
    fn from(e: String) -> Self {
        ImportFailure::Internal(e)
    }
}

impl std::fmt::Display for ImportFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportFailure::Validation(m) => write!(f, "{m}"),
            ImportFailure::Internal(m) => write!(f, "{m}"),
        }
    }
}

#[derive(Debug)]
pub enum ImportError {
    UnknownFormatVersion(u32),
    InvalidMoodScore(u8),
    InvalidRecord(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::UnknownFormatVersion(v) => {
                write!(f, "unknown export format version {v}")
            }
            ImportError::InvalidMoodScore(s) => write!(f, "mood score {s} outside 1-5"),
            ImportError::InvalidRecord(reason) => write!(f, "invalid record: {reason}"),
        }
    }
}

pub async fn import_document(
    user_id: UserId,
    document: ExportDocument,
    mood_source: &dyn OwnMoodRecords,
    mood_sink: &dyn MoodRecordSink,
    practice_source: &dyn OwnPracticeRecords,
    practice_sink: &dyn PracticeSink,
    meditations: &dyn MeditationLookup,
) -> Result<ImportReport, ImportFailure> {
    validate::validate(&document).map_err(|e| ImportFailure::Validation(e.to_string()))?;

    let existing_moods = mood::existing_mood_keys(user_id, mood_source).await?;
    let all_meditations = meditations.list_all().await?;
    let index = meditation_index::meditation_index(&all_meditations);
    let mut report = ImportReport::default();

    mood::apply_mood_entries(
        user_id,
        document.mood_entries,
        existing_moods,
        mood_sink,
        &mut report,
    )
    .await?;

    let existing_practice = practice_source.list_for_user(user_id).await?;
    practice::apply_practice(
        user_id,
        document.practice,
        &existing_practice,
        &index,
        practice_sink,
        &mut report,
    )
    .await?;

    Ok(report)
}
