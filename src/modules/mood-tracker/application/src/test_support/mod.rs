use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use mood_tracker_domain::{MoodEntry, MoodScore};

use crate::ports::{Clock, MoodEntryRepository};

#[derive(Clone)]
pub struct FixedClock(pub DateTime<Utc>);

#[async_trait]
impl Clock for FixedClock {
    async fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct FakeMoodEntryRepository(pub Arc<Mutex<Vec<MoodEntry>>>);

#[async_trait]
impl MoodEntryRepository for FakeMoodEntryRepository {
    async fn insert(
        &self,
        user_id: UserId,
        score: MoodScore,
        note: Option<String>,
        recorded_at: DateTime<Utc>,
    ) -> Result<MoodEntry, String> {
        let mut entries = self.0.lock().unwrap();
        let id = entries.iter().map(MoodEntry::id).max().unwrap_or(0) + 1;
        let entry = MoodEntry::new(id, user_id, score, note, recorded_at, recorded_at);
        entries.push(entry.clone());
        Ok(entry)
    }

    async fn update(
        &self,
        user_id: UserId,
        entry_id: i64,
        score: MoodScore,
        note: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Result<Option<MoodEntry>, String> {
        let mut entries = self.0.lock().unwrap();
        let Some(index) = entries
            .iter()
            .position(|entry| entry.user_id() == user_id && entry.id() == entry_id)
        else {
            return Ok(None);
        };
        let previous = &entries[index];
        let updated = MoodEntry::new(
            entry_id,
            user_id,
            score,
            note,
            previous.recorded_at(),
            updated_at,
        );
        entries[index] = updated.clone();
        Ok(Some(updated))
    }

    async fn delete(&self, user_id: UserId, entry_id: i64) -> Result<bool, String> {
        let mut entries = self.0.lock().unwrap();
        let Some(index) = entries
            .iter()
            .position(|entry| entry.user_id() == user_id && entry.id() == entry_id)
        else {
            return Ok(false);
        };
        entries.remove(index);
        Ok(true)
    }

    async fn list_all_for_user(&self, user_id: UserId) -> Result<Vec<MoodEntry>, String> {
        let mut entries: Vec<_> = self
            .0
            .lock()
            .unwrap()
            .iter()
            .filter(|entry| entry.user_id() == user_id)
            .cloned()
            .collect();
        entries.sort_by(|left, right| {
            right
                .recorded_at()
                .cmp(&left.recorded_at())
                .then_with(|| right.id().cmp(&left.id()))
        });
        Ok(entries)
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let mut rows = self.0.lock().unwrap();
        let before = rows.len();
        rows.retain(|entry| entry.user_id() != user_id);
        Ok((before - rows.len()) as u64)
    }
}

/// Repository whose every operation fails, to exercise error mapping.
pub struct FailingMoodEntryRepository;

pub const REPOSITORY_FAILURE: &str = "storage offline";

#[async_trait]
impl MoodEntryRepository for FailingMoodEntryRepository {
    async fn insert(
        &self,
        _user_id: UserId,
        _score: MoodScore,
        _note: Option<String>,
        _recorded_at: DateTime<Utc>,
    ) -> Result<MoodEntry, String> {
        Err(REPOSITORY_FAILURE.to_owned())
    }

    async fn update(
        &self,
        _user_id: UserId,
        _entry_id: i64,
        _score: MoodScore,
        _note: Option<String>,
        _updated_at: DateTime<Utc>,
    ) -> Result<Option<MoodEntry>, String> {
        Err(REPOSITORY_FAILURE.to_owned())
    }

    async fn delete(&self, _user_id: UserId, _entry_id: i64) -> Result<bool, String> {
        Err(REPOSITORY_FAILURE.to_owned())
    }

    async fn list_all_for_user(&self, _user_id: UserId) -> Result<Vec<MoodEntry>, String> {
        Err(REPOSITORY_FAILURE.to_owned())
    }

    async fn delete_all_for_user(&self, _user_id: UserId) -> Result<u64, String> {
        Err(REPOSITORY_FAILURE.to_owned())
    }
}
