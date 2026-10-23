/// Edits name, category and bundled-glyph key. `icon_key` replaces the stored
/// value; `None` or blank clears it.
#[derive(Debug, Clone)]
pub struct UpdateSound {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub icon_key: Option<String>,
}
