use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimelineKind {
    Mood,
    MeditationNote,
    MeditationSession,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineEvent {
    pub id: i64,
    pub kind: TimelineKind,
    pub title: String,
    pub score: Option<u8>,
    pub note: Option<String>,
    pub recorded_at: DateTime<Utc>,
}
