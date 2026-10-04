use super::mood_record::MoodRecord;
use super::practice_snapshot::PracticeSnapshot;

#[derive(Debug, Clone)]
pub struct ExportDocument {
    pub format_version: u32,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub display_name: Option<String>,
    pub mood_entries: Vec<MoodRecord>,
    pub practice: PracticeSnapshot,
}
