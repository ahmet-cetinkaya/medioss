use portability_application::models::export_document::ExportDocument;
use portability_application::models::mood_record::MoodRecord;

const SUPPORTED_VERSION: u32 = 1;

pub fn to_json(document: &ExportDocument) -> Result<Vec<u8>, String> {
    let payload = JsonDocument {
        format_version: document.format_version,
        exported_at: document.exported_at,
        display_name: document.display_name.clone(),
        mood_entries: document
            .mood_entries
            .iter()
            .map(|e| JsonMoodEntry {
                score: e.score,
                note: e.note.clone(),
                recorded_at: e.recorded_at,
            })
            .collect(),
        practice: JsonPractice {
            sessions: document
                .practice
                .sessions
                .iter()
                .map(|s| JsonSession {
                    meditation_name: s.meditation_name.clone(),
                    meditation_duration_ms: s.meditation_duration_ms,
                    started_at: s.started_at,
                    finished_at: s.finished_at,
                })
                .collect(),
            notes: document
                .practice
                .notes
                .iter()
                .map(|n| JsonNote {
                    meditation_name: n.meditation_name.clone(),
                    meditation_duration_ms: n.meditation_duration_ms,
                    text: n.text.clone(),
                    created_at: n.created_at,
                })
                .collect(),
            playback: document
                .practice
                .playback
                .iter()
                .map(|p| JsonPlayback {
                    meditation_name: p.meditation_name.clone(),
                    meditation_duration_ms: p.meditation_duration_ms,
                    position_ms: p.position_ms,
                    completed: p.completed,
                })
                .collect(),
        },
    };
    serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())
}

pub fn from_json(bytes: &[u8]) -> Result<ExportDocument, String> {
    let payload: JsonDocument =
        serde_json::from_slice(bytes).map_err(|e| format!("not a valid export file: {e}"))?;
    if payload.format_version != SUPPORTED_VERSION {
        return Err(format!(
            "unknown export format version {} (this instance understands {SUPPORTED_VERSION})",
            payload.format_version
        ));
    }
    Ok(ExportDocument {
        format_version: payload.format_version,
        exported_at: payload.exported_at,
        display_name: payload.display_name,
        mood_entries: payload
            .mood_entries
            .into_iter()
            .map(|e| MoodRecord {
                score: e.score,
                note: e.note,
                recorded_at: e.recorded_at,
            })
            .collect(),
        practice: payload.practice.into_practice(),
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonMoodEntry {
    score: u8,
    note: Option<String>,
    recorded_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonSession {
    meditation_name: String,
    meditation_duration_ms: i64,
    started_at: chrono::DateTime<chrono::Utc>,
    finished_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonNote {
    meditation_name: String,
    meditation_duration_ms: i64,
    text: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonPlayback {
    meditation_name: String,
    meditation_duration_ms: i64,
    position_ms: i64,
    completed: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonPractice {
    sessions: Vec<JsonSession>,
    notes: Vec<JsonNote>,
    playback: Vec<JsonPlayback>,
}

impl JsonPractice {
    fn into_practice(self) -> portability_application::models::practice_snapshot::PracticeSnapshot {
        portability_application::models::practice_snapshot::PracticeSnapshot {
            sessions: self
                .sessions
                .into_iter()
                .map(
                    |s| portability_application::models::session_record::SessionRecord {
                        meditation_name: s.meditation_name,
                        meditation_duration_ms: s.meditation_duration_ms,
                        started_at: s.started_at,
                        finished_at: s.finished_at,
                    },
                )
                .collect(),
            notes: self
                .notes
                .into_iter()
                .map(
                    |n| portability_application::models::note_record::NoteRecord {
                        meditation_name: n.meditation_name,
                        meditation_duration_ms: n.meditation_duration_ms,
                        text: n.text,
                        created_at: n.created_at,
                    },
                )
                .collect(),
            playback: self
                .playback
                .into_iter()
                .map(
                    |p| portability_application::models::playback_record::PlaybackRecord {
                        meditation_name: p.meditation_name,
                        meditation_duration_ms: p.meditation_duration_ms,
                        position_ms: p.position_ms,
                        completed: p.completed,
                    },
                )
                .collect(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonDocument {
    format_version: u32,
    exported_at: chrono::DateTime<chrono::Utc>,
    display_name: Option<String>,
    mood_entries: Vec<JsonMoodEntry>,
    practice: JsonPractice,
}

#[cfg(test)]
#[path = "export_format.tests.rs"]
mod tests;
