#[derive(Debug, Clone)]
pub struct PlaybackRecord {
    pub meditation_name: String,
    pub meditation_duration_ms: i64,
    pub position_ms: i64,
    pub completed: bool,
}
