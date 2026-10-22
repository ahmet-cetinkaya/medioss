use crate::models::export_document::ExportDocument;
use crate::models::mood_record::MoodRecord;
use crate::models::note_record::NoteRecord;
use crate::models::playback_record::PlaybackRecord;
use crate::models::practice_snapshot::PracticeSnapshot;
use crate::models::session_record::SessionRecord;
use chrono::{DateTime, Utc};
use kernel::UserId;

pub fn user() -> UserId {
    UserId::new(7)
}

pub fn ts(minutes_ago: i64) -> DateTime<Utc> {
    Utc::now() - chrono::Duration::minutes(minutes_ago)
}

pub fn entry(score: u8, note: Option<&str>, minutes_ago: i64) -> MoodRecord {
    MoodRecord {
        score,
        note: note.map(String::from),
        recorded_at: ts(minutes_ago),
    }
}

pub fn snapshot(
    sessions: Vec<SessionRecord>,
    notes: Vec<NoteRecord>,
    playback: Vec<PlaybackRecord>,
) -> PracticeSnapshot {
    PracticeSnapshot {
        sessions,
        notes,
        playback,
    }
}

pub fn document(mood: Vec<MoodRecord>, practice: PracticeSnapshot) -> ExportDocument {
    ExportDocument {
        format_version: 1,
        exported_at: Utc::now(),
        display_name: Some("Test User".into()),
        mood_entries: mood,
        practice,
    }
}

pub fn library_meditations() -> Vec<crate::ports::meditation_lookup::MeditationRef> {
    use crate::ports::meditation_lookup::MeditationRef;
    vec![
        MeditationRef {
            id: 1,
            name: "Café Quietude".into(),
            duration_ms: 120_000,
        },
        MeditationRef {
            id: 2,
            name: "Evening Calm".into(),
            duration_ms: 300_000,
        },
    ]
}
