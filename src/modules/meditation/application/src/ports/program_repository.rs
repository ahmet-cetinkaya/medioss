use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// A program: an ordered collection of meditations, optionally with a cover
/// image and category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub id: i64,
    pub title: String,
    /// Storage folder name (slug + program-unique suffix), fixed at creation.
    pub folder: String,
    pub description: Option<String>,
    pub cover_storage_key: Option<String>,
    pub category_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Persists programs and their memberships.
#[async_trait]
pub trait ProgramRepository: Send + Sync {
    /// Inserts a program row, assigning its identifier and storage folder.
    async fn insert(
        &self,
        title: String,
        description: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Program, String>;

    /// Records the stored cover location; `false` when the program is unknown.
    async fn set_cover_key(&self, id: i64, key: &str) -> Result<bool, String>;

    /// Finds a program by identifier.
    async fn find_by_id(&self, id: i64) -> Result<Option<Program>, String>;

    /// Lists every program.
    async fn list_all(&self) -> Result<Vec<Program>, String>;

    /// Updates title and description; `false` when the identifier is unknown.
    async fn update(
        &self,
        id: i64,
        title: String,
        description: Option<String>,
    ) -> Result<bool, String>;

    /// Deletes a program and returns it (with its cover key) so the caller can
    /// clean up the cover bytes; `None` when the identifier is unknown.
    async fn delete(&self, id: i64) -> Result<Option<Program>, String>;

    /// Assigns or clears a program's category; `false` when the program is
    /// unknown.
    async fn set_category(&self, id: i64, category_id: Option<i64>) -> Result<bool, String>;

    /// Clears the category from every program that held it; returns the number
    /// of programs affected.
    async fn clear_category(&self, category_id: i64) -> Result<u64, String>;
}
