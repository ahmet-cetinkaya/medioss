use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// A category programs can be grouped under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Persists program categories.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Inserts a category row, assigning its identifier. Callers check for
    /// duplicates with [`CategoryRepository::find_by_name`] first.
    async fn insert(&self, name: String, created_at: DateTime<Utc>) -> Result<Category, String>;

    /// Finds a category by its exact name.
    async fn find_by_name(&self, name: &str) -> Result<Option<Category>, String>;

    /// Finds a category by identifier.
    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, String>;

    /// Lists every category.
    async fn list_all(&self) -> Result<Vec<Category>, String>;

    /// Renames a category; `false` when the identifier is unknown.
    async fn rename(
        &self,
        id: i64,
        name: String,
        updated_at: DateTime<Utc>,
    ) -> Result<bool, String>;

    /// Deletes a category; `false` when the identifier is unknown.
    async fn delete(&self, id: i64) -> Result<bool, String>;
}
