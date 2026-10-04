#[cfg(test)]
#[path = "import.tests.rs"]
mod import_tests;

use crate::models::export_document::ExportDocument;
use crate::models::import_report::ImportReport;
use crate::ports::meditation_lookup::{MeditationLookup, MeditationRef};
use crate::ports::mood_record_sink::MoodRecordSink;
use crate::ports::own_mood_records::OwnMoodRecords;
use crate::ports::own_practice_records::OwnPracticeRecords;
use crate::ports::practice_sink::PracticeSink;
use chrono::{DateTime, Utc};
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

fn validate(document: &ExportDocument) -> Result<(), ImportError> {
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

fn meditation_index(meditations: &[MeditationRef]) -> std::collections::HashMap<(&str, i64), i64> {
    let mut index = std::collections::HashMap::new();
    for m in meditations {
        index
            .entry((m.name.as_str(), m.duration_ms))
            .and_modify(|current: &mut i64| *current = (*current).min(m.id))
            .or_insert(m.id);
    }
    index
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
    validate(&document).map_err(|e| ImportFailure::Validation(e.to_string()))?;

    let mut existing: std::collections::HashSet<(DateTime<Utc>, u8, Option<String>)> = mood_source
        .list_for_user(user_id)
        .await?
        .into_iter()
        .map(|r| (r.recorded_at, r.score, r.note))
        .collect();

    let all_meditations = meditations.list_all().await?;
    let index = meditation_index(&all_meditations);
    let mut report = ImportReport::default();

    for entry in document.mood_entries {
        let key = (entry.recorded_at, entry.score, entry.note.clone());
        if existing.contains(&key) {
            report.mood_duplicates_skipped += 1;
        } else {
            mood_sink.insert(user_id, entry).await?;
            existing.insert(key);
            report.mood_created += 1;
        }
    }

    let existing_practice = practice_source.list_for_user(user_id).await?;
    let session_keys: std::collections::HashSet<_> = existing_practice
        .sessions
        .iter()
        .map(|s| {
            (
                s.meditation_name.as_str(),
                s.meditation_duration_ms,
                s.started_at,
            )
        })
        .collect();
    let note_keys: std::collections::HashSet<_> = existing_practice
        .notes
        .iter()
        .map(|n| {
            (
                n.meditation_name.as_str(),
                n.meditation_duration_ms,
                n.text.as_str(),
                n.created_at,
            )
        })
        .collect();
    let playback_keys: std::collections::HashSet<_> = existing_practice
        .playback
        .iter()
        .map(|p| {
            (
                p.meditation_name.as_str(),
                p.meditation_duration_ms,
                p.position_ms,
                p.completed,
            )
        })
        .collect();

    let link =
        |name: &str, duration_ms: i64| -> Option<i64> { index.get(&(name, duration_ms)).copied() };

    for session in document.practice.sessions {
        let dup = session_keys.contains(&(
            session.meditation_name.as_str(),
            session.meditation_duration_ms,
            session.started_at,
        ));
        match link(&session.meditation_name, session.meditation_duration_ms) {
            Some(id) if !dup => {
                practice_sink
                    .record_session(user_id, id, session.started_at, session.finished_at)
                    .await
                    .map_err(ImportFailure::Internal)?;
                report.practice_linked += 1;
            }
            Some(_) => report.practice_duplicates_skipped += 1,
            None => report.practice_skipped += 1,
        }
    }
    for note in document.practice.notes {
        let dup = note_keys.contains(&(
            note.meditation_name.as_str(),
            note.meditation_duration_ms,
            note.text.as_str(),
            note.created_at,
        ));
        match link(&note.meditation_name, note.meditation_duration_ms) {
            Some(id) if !dup => {
                practice_sink
                    .insert_note(user_id, id, &note.text, note.created_at)
                    .await
                    .map_err(ImportFailure::Internal)?;
                report.practice_linked += 1;
            }
            Some(_) => report.practice_duplicates_skipped += 1,
            None => report.practice_skipped += 1,
        }
    }
    for playback in document.practice.playback {
        let dup = playback_keys.contains(&(
            playback.meditation_name.as_str(),
            playback.meditation_duration_ms,
            playback.position_ms,
            playback.completed,
        ));
        match link(&playback.meditation_name, playback.meditation_duration_ms) {
            Some(id) if !dup => {
                practice_sink
                    .upsert_position(user_id, id, playback.position_ms)
                    .await?;
                if playback.completed {
                    practice_sink.set_completed(user_id, id, true).await?;
                }
                report.practice_linked += 1;
            }
            Some(_) => report.practice_duplicates_skipped += 1,
            None => report.practice_skipped += 1,
        }
    }

    Ok(report)
}
