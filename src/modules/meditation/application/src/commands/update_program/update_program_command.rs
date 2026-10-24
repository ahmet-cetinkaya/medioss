#[derive(Debug, Clone)]
pub struct UpdateProgramCommand {
    pub program_id: i64,
    pub title: String,
    pub description: Option<String>,
}
