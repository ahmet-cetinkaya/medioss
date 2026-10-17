use chrono::{DateTime, Utc};
use kernel::UserId;

use super::repo_state::RepoState;
use crate::ports::{MeditationSession, SessionRepository};

/// SessionRepository fake honoring the (user, meditation, started_at) key.
#[derive(Clone)]
pub struct FakeSessionRepository(pub RepoState);

#[async_trait::async_trait]
impl SessionRepository for FakeSessionRepository {
    async fn record_if_new(
        &self,
        user_id: UserId,
        meditation_id: i64,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
    ) -> Result<MeditationSession, String> {
        let mut rows = self.0.sessions.lock().unwrap();
        if let Some(existing) = rows.iter().find(|session| {
            session.user_id == user_id
                && session.meditation_id == meditation_id
                && session.started_at == started_at
        }) {
            return Ok(existing.clone());
        }
        let id = RepoState::next_id(&rows, |session| session.id);
        let session = MeditationSession {
            id,
            user_id,
            meditation_id,
            started_at,
            finished_at,
        };
        rows.push(session.clone());
        Ok(session)
    }

    async fn count_for(&self, user_id: UserId, meditation_id: i64) -> Result<i64, String> {
        Ok(self
            .0
            .sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|session| session.user_id == user_id && session.meditation_id == meditation_id)
            .count() as i64)
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<MeditationSession>, String> {
        Ok(self
            .0
            .sessions
            .lock()
            .unwrap()
            .iter()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete_all_for_user(&self, user_id: UserId) -> Result<u64, String> {
        let mut rows = self.0.sessions.lock().unwrap();
        let before = rows.len();
        rows.retain(|session| session.user_id != user_id);
        Ok((before - rows.len()) as u64)
    }
}
