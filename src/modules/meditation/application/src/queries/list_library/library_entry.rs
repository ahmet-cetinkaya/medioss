use meditation_domain::Meditation;

/// A program a library meditation belongs to, as a label for list cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryProgram {
    pub id: i64,
    pub title: String,
}

/// A library meditation decorated with its home program, so the client can
/// render program links without a second request.
#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub meditation: Meditation,
    pub program_id: Option<i64>,
    pub program_has_cover: bool,
    /// Every program the meditation belongs to, earliest membership first.
    pub programs: Vec<LibraryProgram>,
}
