#[derive(Debug, Clone)]
pub struct UpdateProgramOrderCommand {
    pub program_id: i64,
    pub meditation_ids: Vec<i64>,
}
