use crate::ports::Program;

pub struct DailySuggestion {
    pub meditation: meditation_domain::Meditation,
    pub program: Option<Program>,
}
