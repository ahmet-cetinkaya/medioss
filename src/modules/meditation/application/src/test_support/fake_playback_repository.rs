use chrono::{DateTime, Utc};
use kernel::UserId;

use super::repo_state::RepoState;
use crate::ports::{PlaybackState, PlaybackStateRepository};

/// PlaybackStateRepository fake.
#[derive(Clone)]
pub struct FakePlaybackRepository(pub RepoState);

impl FakePlaybackRepository {
    fn touch(&self, user_id: UserId, meditation_id: i64) {
        let mut touches = self.0.playback_touches.lock().unwrap();
        touches.retain(|(existing_user, existing_meditation, _)| {
            !(*existing_user == user_id.value() && *existing_meditation == meditation_id)
        });
        touches.push((user_id.value(), meditation_id, Utc::now()));
    }
}

#[async_trait::async_trait]
impl PlaybackStateRepository for FakePlaybackRepository {
    async fn upsert_position(
        &self,
        user_id: UserId,
        meditation_id: i64,
        position_ms: i64,
    ) -> Result<(), String> {
        let mut rows = self.0.playback.lock().unwrap();
        let state = rows
            .entry((user_id.value(), meditation_id))
            .or_insert(PlaybackState {
                position_ms: 0,
                completed: false,
            });
        state.position_ms = position_ms;
        self.touch(user_id, meditation_id);
        Ok(())
    }

    async fn set_completed(
        &self,
        user_id: UserId,
        meditation_id: i64,
        completed: bool,
    ) -> Result<(), String> {
        let mut rows = self.0.playback.lock().unwrap();
        let state = rows
            .entry((user_id.value(), meditation_id))
            .or_insert(PlaybackState {
                position_ms: 0,
                completed: false,
            });
        state.completed = completed;
        self.touch(user_id, meditation_id);
        Ok(())
    }

    async fn reset_position(&self, user_id: UserId, meditation_id: i64) -> Result<(), String> {
        let mut rows = self.0.playback.lock().unwrap();
        if let Some(state) = rows.get_mut(&(user_id.value(), meditation_id)) {
            state.position_ms = 0;
        }
        Ok(())
    }

    async fn get(
        &self,
        user_id: UserId,
        meditation_id: i64,
    ) -> Result<Option<PlaybackState>, String> {
        Ok(self
            .0
            .playback
            .lock()
            .unwrap()
            .get(&(user_id.value(), meditation_id))
            .copied())
    }

    async fn list_touched(&self, user_id: UserId) -> Result<Vec<(i64, DateTime<Utc>)>, String> {
        Ok(self
            .0
            .playback_touches
            .lock()
            .unwrap()
            .iter()
            .rev()
            .filter(|(touched_user, _, _)| *touched_user == user_id.value())
            .map(|(_, meditation_id, touched_at)| (*meditation_id, *touched_at))
            .collect())
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let mut rows = self.0.playback.lock().unwrap();
        let before = rows.len();
        rows.retain(|key, _| key.0 != user_id.value());
        let mut touches = self.0.playback_touches.lock().unwrap();
        touches.retain(|(existing_user, _, _)| *existing_user != user_id.value());
        Ok((before - rows.len()) as u64)
    }
}
