#[derive(Debug, Clone)]
pub struct NoteRecord {
    pub meditation_name: String,
    pub meditation_duration_ms: i64,
    pub text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
