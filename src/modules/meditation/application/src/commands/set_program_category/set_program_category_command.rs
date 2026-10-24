#[derive(Debug, Clone)]
pub struct SetProgramCategoryCommand {
    pub program_id: i64,
    pub category_id: Option<i64>,
}
