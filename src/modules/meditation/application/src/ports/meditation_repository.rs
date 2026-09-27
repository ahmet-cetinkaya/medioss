use async_trait::async_trait;
use meditation_domain::Meditation;

/// Persists the shared meditation library rows.
#[async_trait]
pub trait MeditationRepository: Send + Sync {
    /// Inserts a new meditation row, assigning its identifier.
    async fn insert(
        &self,
        name: String,
        content_type: &str,
        size_bytes: i64,
        storage_key: String,
        duration_ms: i64,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Meditation, String>;

    /// Finds a meditation by identifier.
    async fn find_by_id(&self, id: i64) -> Result<Option<Meditation>, String>;

    /// Lists every meditation, newest first (`created_at` DESC, then id DESC).
    async fn list_all(&self) -> Result<Vec<Meditation>, String>;

    /// Lists every meditation whose storage key starts with the given
    /// folder prefix (e.g. `"{folder}/"`) — used to find every file that
    /// actually lives in a program's folder on disk, independent of
    /// membership rows, so a deleted program's folder is never removed
    /// while a meditation's bytes still live in it.
    async fn list_by_storage_prefix(&self, prefix: &str) -> Result<Vec<Meditation>, String>;

    /// Deletes a meditation row; `false` when the identifier is unknown.
    async fn delete(&self, id: i64) -> Result<bool, String>;

    /// Updates the stored location after a file relocation.
    async fn set_storage_key(&self, id: i64, key: &str) -> Result<(), String>;
}
