#[derive(Debug, Clone)]
pub struct UploadMeditationCommand {
    pub name: String,
    pub program_id: i64,
    pub bytes: Vec<u8>,
    pub description: Option<String>,
}

/// Upload size cap: 100MB applies to the original bytes (spec FR-001/002).
pub const MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;
