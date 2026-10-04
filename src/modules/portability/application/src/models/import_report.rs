#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct ImportReport {
    pub mood_created: u32,
    pub mood_duplicates_skipped: u32,
    pub practice_linked: u32,
    pub practice_skipped: u32,
}
