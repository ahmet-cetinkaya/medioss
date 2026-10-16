use chrono::{DateTime, Utc};
use kernel::UserId;

use super::repo_state::RepoState;
use crate::ports::{Note, NoteRepository};

/// NoteRepository fake with author scoping.
#[derive(Clone)]
pub struct FakeNoteRepository(pub RepoState);

#[async_trait::async_trait]
impl NoteRepository for FakeNoteRepository {
    async fn insert(
        &self,
        user_id: UserId,
        meditation_id: i64,
        text: String,
        created_at: DateTime<Utc>,
    ) -> Result<Note, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let id = RepoState::next_id(&rows, |note| note.id);
        let note = Note {
            id,
            user_id,
            meditation_id,
            text,
            created_at,
            updated_at: created_at,
        };
        rows.push(note.clone());
        Ok(note)
    }

    async fn list_for(&self, user_id: UserId, meditation_id: i64) -> Result<Vec<Note>, String> {
        let mut rows: Vec<Note> = self
            .0
            .notes
            .lock()
            .unwrap()
            .iter()
            .filter(|note| note.user_id == user_id && note.meditation_id == meditation_id)
            .cloned()
            .collect();
        rows.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(rows)
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Note>, String> {
        let mut rows: Vec<Note> = self
            .0
            .notes
            .lock()
            .unwrap()
            .iter()
            .filter(|note| note.user_id == user_id)
            .cloned()
            .collect();
        rows.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        Ok(rows)
    }

    async fn update(
        &self,
        note_id: i64,
        meditation_id: i64,
        user_id: UserId,
        text: String,
        updated_at: DateTime<Utc>,
    ) -> Result<bool, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let Some(note) = rows.iter_mut().find(|note| {
            note.id == note_id && note.user_id == user_id && note.meditation_id == meditation_id
        }) else {
            return Ok(false);
        };
        note.text = text;
        note.updated_at = updated_at;
        Ok(true)
    }

    async fn delete(
        &self,
        note_id: i64,
        meditation_id: i64,
        user_id: UserId,
    ) -> Result<bool, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let len = rows.len();
        rows.retain(|note| {
            !(note.id == note_id && note.user_id == user_id && note.meditation_id == meditation_id)
        });
        Ok(rows.len() != len)
    }

    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let len = rows.len();
        rows.retain(|note| note.meditation_id != meditation_id);
        Ok((len - rows.len()) as u64)
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let mut rows = self.0.notes.lock().unwrap();
        let len = rows.len();
        rows.retain(|note| note.user_id != user_id);
        Ok((len - rows.len()) as u64)
    }
}
