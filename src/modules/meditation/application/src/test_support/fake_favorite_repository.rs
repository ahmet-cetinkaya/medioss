use std::collections::HashSet;

use chrono::{DateTime, Utc};
use kernel::UserId;

use super::repo_state::RepoState;
use crate::ports::FavoriteRepository;

/// FavoriteRepository fake with user scoping.
#[derive(Clone)]
pub struct FakeFavoriteRepository(pub RepoState);

#[async_trait::async_trait]
impl FavoriteRepository for FakeFavoriteRepository {
    async fn add(
        &self,
        user_id: UserId,
        meditation_id: i64,
        created_at: DateTime<Utc>,
    ) -> Result<(), String> {
        let mut rows = self.0.favorites.lock().unwrap();
        let exists = rows
            .iter()
            .any(|(user, meditation, _)| *user == user_id.value() && *meditation == meditation_id);
        if !exists {
            rows.push((user_id.value(), meditation_id, created_at));
        }
        Ok(())
    }

    async fn remove(&self, user_id: UserId, meditation_id: i64) -> Result<bool, String> {
        let mut rows = self.0.favorites.lock().unwrap();
        let before = rows.len();
        rows.retain(|(user, meditation, _)| {
            !(*user == user_id.value() && *meditation == meditation_id)
        });
        Ok(rows.len() < before)
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<i64>, String> {
        let mut rows: Vec<_> = self
            .0
            .favorites
            .lock()
            .unwrap()
            .iter()
            .filter(|(user, _, _)| *user == user_id.value())
            .copied()
            .collect();
        rows.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| right.1.cmp(&left.1)));
        Ok(rows
            .into_iter()
            .map(|(_, meditation, _)| meditation)
            .collect())
    }

    async fn ids_for_user(&self, user_id: UserId) -> Result<HashSet<i64>, String> {
        Ok(self
            .0
            .favorites
            .lock()
            .unwrap()
            .iter()
            .filter(|(user, _, _)| *user == user_id.value())
            .map(|(_, meditation, _)| *meditation)
            .collect())
    }

    async fn delete_for_meditation(&self, meditation_id: i64) -> Result<u64, String> {
        let mut rows = self.0.favorites.lock().unwrap();
        let before = rows.len();
        rows.retain(|(_, meditation, _)| *meditation != meditation_id);
        Ok((before - rows.len()) as u64)
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let mut rows = self.0.favorites.lock().unwrap();
        let before = rows.len();
        rows.retain(|(user, _, _)| *user != user_id.value());
        Ok((before - rows.len()) as u64)
    }
}
