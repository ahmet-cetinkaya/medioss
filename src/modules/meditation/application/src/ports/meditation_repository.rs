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
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Meditation, String>;

    /// Finds a meditation by identifier.
    async fn find_by_id(&self, id: i64) -> Result<Option<Meditation>, String>;

    /// Lists every meditation, newest first (`created_at` DESC, then id DESC).
    async fn list_all(&self) -> Result<Vec<Meditation>, String>;

    /// Deletes a meditation row; `false` when the identifier is unknown.
    async fn delete(&self, id: i64) -> Result<bool, String>;

    /// Updates the stored location after a file relocation.
    async fn set_storage_key(&self, id: i64, key: &str) -> Result<(), String>;
}
