use meditation_domain::Meditation;

/// A library meditation decorated with its home program, so the client can
/// render program links without a second request.
#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub meditation: Meditation,
    pub program_id: Option<i64>,
    pub program_has_cover: bool,
}
