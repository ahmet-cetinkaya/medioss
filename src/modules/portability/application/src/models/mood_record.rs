#[derive(Debug, Clone)]
pub struct MoodRecord {
    pub score: u8,
    pub note: Option<String>,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}
