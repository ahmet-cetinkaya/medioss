use async_trait::async_trait;
use chrono::{DateTime, Utc};
use identity_domain::FailedAttempts;

use super::{fake_repository::FakeRepository, fake_repository_state::FakeRepositoryState};
use crate::ports::{LoginThrottleRepository, RepositoryError};

#[async_trait]
impl LoginThrottleRepository for FakeRepository {
    async fn failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<Option<FailedAttempts>, RepositoryError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .failed_attempts
            .get(normalized_identifier)
            .cloned())
    }

    async fn record_failed_attempt(
        &self,
        normalized_identifier: &str,
        attempted_at: DateTime<Utc>,
    ) -> Result<FailedAttempts, RepositoryError> {
        let mut state = self.state.lock().unwrap();
        let attempts = state
            .failed_attempts
            .get(normalized_identifier)
            .map_or_else(
                || FailedAttempts::first_failure(attempted_at),
                |current| current.record_failure(attempted_at),
            );
        let mut failed_attempts = state.failed_attempts.clone();
        failed_attempts.insert(normalized_identifier.to_owned(), attempts.clone());
        *state = FakeRepositoryState {
            failed_attempts,
            ..state.clone()
        };
        Ok(attempts)
    }

    async fn clear_failed_attempts(
        &self,
        normalized_identifier: &str,
    ) -> Result<(), RepositoryError> {
        self.replace_state(|state| {
            let mut failed_attempts = state.failed_attempts.clone();
            failed_attempts.remove(normalized_identifier);
            FakeRepositoryState {
                failed_attempts,
                ..state
            }
        });
        Ok(())
    }
}
