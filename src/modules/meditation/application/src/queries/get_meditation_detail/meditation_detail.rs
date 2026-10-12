use crate::ports::Program;

/// One meditation plus its home program: what the Player screen renders.
pub struct MeditationDetail {
    pub meditation: meditation_domain::Meditation,
    pub program: Option<Program>,
}
