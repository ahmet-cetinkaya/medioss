#[derive(Debug, Clone)]
pub struct RenameCategoryCommand {
    pub category_id: i64,
    pub name: String,
}
