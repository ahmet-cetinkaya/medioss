#[derive(Debug, Clone)]
pub struct UpdateProgram {
    pub program_id: i64,
    pub title: String,
    pub description: Option<String>,
}
