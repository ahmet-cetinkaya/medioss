pub mod media;
pub mod staged_upload;

mod command_handler;
mod query_handler;

pub use command_handler::CommandHandler;
pub use query_handler::QueryHandler;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;
use soundscape_domain::{Mix, Sound, SoundCategory, SoundIcon};

#[derive(Debug, Clone)]
pub struct NewSound {
    pub name: String,
    pub category_id: i64,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_key: String,
    pub duration_ms: i64,
    pub created_at: DateTime<Utc>,
    pub icon_key: Option<String>,
}
#[derive(Debug, Clone)]
pub struct NewMix {
    pub user_id: UserId,
    pub name: String,
    pub items: Vec<soundscape_domain::MixItem>,
    pub now: DateTime<Utc>,
}

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<SoundCategory>, String>;
    async fn find(&self, id: i64) -> Result<Option<SoundCategory>, String>;
    async fn create(&self, name: &str) -> Result<SoundCategory, String>;
    async fn rename(&self, id: i64, name: &str) -> Result<Option<SoundCategory>, String>;
    async fn delete_empty(&self, id: i64) -> Result<CategoryDelete, String>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryDelete {
    Deleted,
    NotFound,
    InUse,
}

#[async_trait]
pub trait SoundRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Sound>, String>;
    async fn find(&self, id: i64) -> Result<Option<Sound>, String>;
    async fn create(&self, sound: NewSound) -> Result<Sound, String>;
    async fn update(
        &self,
        id: i64,
        name: &str,
        category_id: i64,
        icon_key: Option<&str>,
    ) -> Result<Option<Sound>, String>;
    /// Sets or clears (`None`) the uploaded icon image location.
    async fn set_icon(&self, id: i64, icon: Option<SoundIcon>) -> Result<Option<Sound>, String>;
    async fn delete_with_mix_items(&self, id: i64) -> Result<Option<Sound>, String>;
}

#[async_trait]
pub trait MixRepository: Send + Sync {
    async fn list_for(&self, user_id: UserId) -> Result<Vec<Mix>, String>;
    async fn find_for(&self, id: i64, user_id: UserId) -> Result<Option<Mix>, String>;
    async fn create(&self, mix: NewMix) -> Result<Mix, String>;
    async fn replace(&self, id: i64, mix: NewMix) -> Result<Option<Mix>, String>;
    async fn delete_for(&self, id: i64, user_id: UserId) -> Result<bool, String>;

    /// Deletes every mix of one user, including mix items (account
    /// deletion path).
    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String>;
}

#[async_trait]
pub trait Clock: Send + Sync {
    async fn now(&self) -> DateTime<Utc>;
}
