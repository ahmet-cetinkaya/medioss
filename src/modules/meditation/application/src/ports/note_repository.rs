use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;

/// A user's practice note on one meditation, private to its author.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: i64,
    pub user_id: UserId,
    pub meditation_id: i64,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Persists practice notes. Every read and write is author-scoped: another
/// user's note is indistinguishable from a nonexistent one (spec FR-037).
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// Inserts a note, assigning its identifier.
    async fn insert(
        &self,
        user_id: UserId,
        meditation_id: i64,
        text: String,
        created_at: DateTime<Utc>,
    ) -> Result<Note, String>;

    /// Lists the author's notes for one meditation, newest first.
    async fn list_for(&self, user_id: UserId, meditation_id: i64) -> Result<Vec<Note>, String>;

    /// Lists every note owned by one author, newest first.
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<Note>, String>;

    /// Updates the author's note on the named meditation; `false` when
    /// unknown, not the author, or belonging to a different meditation.
    async fn update(
        &self,
        note_id: i64,
        meditation_id: i64,
        user_id: UserId,
        text: String,
        updated_at: DateTime<Utc>,
    ) -> Result<bool, String>;

    /// Deletes the author's note on the named meditation; `false` when
    /// unknown, not the author, or belonging to a different meditation.
    async fn delete(
        &self,
        note_id: i64,
        meditation_id: i64,
        user_id: UserId,
    ) -> Result<bool, String>;

    /// Deletes every note on a meditation (admin meditation delete);
    /// returns the number removed.
    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String>;
}
