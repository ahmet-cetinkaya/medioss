#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub meditation_name: String,
    pub meditation_duration_ms: i64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: chrono::DateTime<chrono::Utc>,
}
