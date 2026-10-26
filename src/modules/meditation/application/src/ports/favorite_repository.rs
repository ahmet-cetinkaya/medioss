use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use kernel::UserId;

/// Persists each user's favorite meditations. Every read and write is
/// user-scoped: one user's favorites are invisible to every other user.
#[async_trait]
pub trait FavoriteRepository: Send + Sync {
    /// Marks a meditation as a favorite; a repeat call keeps the original
    /// favorited-at time and still succeeds.
    async fn add(
        &self,
        user_id: UserId,
        meditation_id: i64,
        created_at: DateTime<Utc>,
    ) -> Result<(), String>;

    /// Removes a favorite; `false` when it was not a favorite.
    async fn remove(&self, user_id: UserId, meditation_id: i64) -> Result<bool, String>;

    /// Lists the user's favorite meditation identifiers, most recently
    /// favorited first (`created_at` DESC, then meditation id DESC).
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<i64>, String>;

    /// The user's favorite meditation identifiers as a set, for decorating
    /// many meditations with one read.
    async fn ids_for_user(&self, user_id: UserId) -> Result<HashSet<i64>, String>;

    /// Deletes every favorite of one meditation (admin meditation delete);
    /// returns the number removed.
    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String>;

    /// Deletes every favorite of one user (account deletion path); returns
    /// the number removed.
    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String>;
}
