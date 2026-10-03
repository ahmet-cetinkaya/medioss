use chrono::{DateTime, Utc};

use crate::ports::Program;

pub struct RecentProgram {
    pub program: Program,
    /// When the caller most recently touched this program \u2014 feeds the
    /// history timeline's date grouping.
    pub last_touched_at: DateTime<Utc>,
}
