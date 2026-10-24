use chrono::{DateTime, Utc};
use kernel::UserId;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("mood score must be between 1 and 5")]
pub struct MoodScoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MoodScore(u8);

impl MoodScore {
    pub fn new(value: u8) -> Result<Self, MoodScoreError> {
        if (1..=5).contains(&value) {
            Ok(Self(value))
        } else {
            Err(MoodScoreError)
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoodEntry {
    id: i64,
    user_id: UserId,
    score: MoodScore,
    note: Option<String>,
    recorded_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl MoodEntry {
    pub fn new(
        id: i64,
        user_id: UserId,
        score: MoodScore,
        note: Option<String>,
        recorded_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            score,
            note,
            recorded_at,
            updated_at,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn score(&self) -> MoodScore {
        self.score
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
#[path = "mood_entry.tests.rs"]
mod tests;
