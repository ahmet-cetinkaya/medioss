use super::ImportError;
use crate::models::export_document::ExportDocument;
use chrono::Utc;

pub(super) fn validate(document: &ExportDocument) -> Result<(), ImportError> {
    if document.format_version != crate::export::FORMAT_VERSION {
        return Err(ImportError::UnknownFormatVersion(document.format_version));
    }
    for e in &document.mood_entries {
        if !(1..=5).contains(&e.score) {
            return Err(ImportError::InvalidMoodScore(e.score));
        }
        if e.recorded_at > Utc::now() {
            return Err(ImportError::InvalidRecord(
                "mood entry recorded in the future".into(),
            ));
        }
        if let Some(note) = &e.note {
            if note.chars().count() > 10_000 {
                return Err(ImportError::InvalidRecord(
                    "mood note must be at most 10000 characters".into(),
                ));
            }
        }
    }
    for n in &document.practice.notes {
        if n.text.trim().is_empty() {
            return Err(ImportError::InvalidRecord(
                "note text must not be empty".into(),
            ));
        }
    }
    for s in &document.practice.sessions {
        if s.finished_at < s.started_at {
            return Err(ImportError::InvalidRecord(
                "session finishes before it starts".into(),
            ));
        }
        if s.meditation_duration_ms < 0 || s.meditation_duration_ms > 24 * 60 * 60 * 1000 {
            return Err(ImportError::InvalidRecord("implausible duration".into()));
        }
        if s.meditation_name.trim().is_empty() {
            return Err(ImportError::InvalidRecord("empty meditation name".into()));
        }
    }
    Ok(())
}
