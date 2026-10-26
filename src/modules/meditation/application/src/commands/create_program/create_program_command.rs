#[derive(Debug, Clone)]
pub struct CreateProgramCommand {
    pub title: String,
    pub description: Option<String>,
    pub cover_bytes: Option<Vec<u8>>,
}
