#[derive(Debug, Clone)]
pub struct CreateProgram {
    pub title: String,
    pub description: Option<String>,
    pub cover_bytes: Option<Vec<u8>>,
}

/// Cover size cap: 5MB (spec FR-026).
pub const MAX_COVER_BYTES: usize = 5 * 1024 * 1024;
