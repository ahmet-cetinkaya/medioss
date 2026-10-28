/// A program a favorite meditation belongs to, as a label for the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteProgram {
    pub id: i64,
    pub title: String,
}
