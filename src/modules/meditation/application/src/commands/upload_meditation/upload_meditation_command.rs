#[derive(Debug, Clone)]
pub struct UploadMeditationCommand {
    pub name: String,
    pub program_id: i64,
    pub bytes: Vec<u8>,
    pub description: Option<String>,
}
